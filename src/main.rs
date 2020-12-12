#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;

mod db;
mod homectl_core;
mod integrations;

// use db::{actions::find_floorplans, establish_connection};
use anyhow::{Context, Result};
use homectl_core::{
    devices::Devices,
    events::*,
    groups::Groups,
    integrations::Integrations,
    rules::Rules,
    scene::{CycleScenesDescriptor, SceneDescriptor},
    scenes::Scenes,
};
use std::{error::Error, sync::{Arc, Mutex}};

struct State {
    integrations: Arc<Mutex<Integrations>>,
    groups: Arc<Mutex<Groups>>,
    scenes: Arc<Mutex<Scenes>>,
    devices: Arc<Mutex<Devices>>,
    rules: Arc<Mutex<Rules>>,
}

// https://github.com/actix/examples/blob/master/diesel/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (config, opaque_integrations_configs) = homectl_core::config::read_config()?;

    // println!("Using config:");
    // println!("{:#?}", config);

    let (sender, receiver) = mk_channel();

    let integrations = Integrations::new(sender.clone());
    let groups = Arc::new(Mutex::new(Groups::new(config.groups)));
    let scenes = Arc::new(Mutex::new(Scenes::new(config.scenes, Arc::clone(&groups))));
    let devices = Devices::new(sender.clone(), Arc::clone(&scenes));
    let rules = Rules::new(config.routines, sender.clone());

    for (id, integration_config) in &config.integrations {
        let opaque_integration_config: &config::Value = opaque_integrations_configs
            .get(id)
            .with_context(|| format!("Expected to find config for integration with id {}", id))?;

        integrations
            .load_integration(&integration_config.plugin, id, opaque_integration_config)
            .await?;
    }

    let _: Result<()> = {
        integrations.run_register_pass().await?;
        integrations.run_start_pass().await?;

        Ok(())
    };

    let state = State {
        integrations: Arc::new(Mutex::new(integrations)),
        groups,
        scenes,
        devices: Arc::new(Mutex::new(devices)),
        rules: Arc::new(Mutex::new(rules)),
    };

    loop {
        let msg = receiver.recv().await?;

        // println!("got msg: {:?}", msg);

        let result: Result<()> = match &msg {
            Message::IntegrationDeviceRefresh { device } => {
                let mut devices = state.devices.lock().unwrap();
                devices.handle_integration_device_refresh(device).await;
                Ok(())
            }
            Message::DeviceUpdate {
                old_state,
                new_state,
                old,
                new,
            } => {
                let rules = state.rules.lock().unwrap();
                rules
                    .handle_device_update(old_state, new_state, old, new)
                    .await;
                Ok(())
            }
            Message::SetDeviceState { device } => {
                let mut devices = state.devices.lock().unwrap();
                devices.set_device_state(&device, false).await;
                Ok(())
            }
            Message::SetIntegrationDeviceState { device } => {
                let integrations = state.integrations.lock().unwrap();
                integrations.set_integration_device_state(device).await
            }
            Message::ActivateScene(SceneDescriptor { scene_id }) => {
                let mut devices = state.devices.lock().unwrap();
                devices.activate_scene(&scene_id).await;
                Ok(())
            }
            Message::CycleScenes(CycleScenesDescriptor { scenes }) => {
                let mut devices = state.devices.lock().unwrap();
                devices.cycle_scenes(&scenes).await;
                Ok(())
            }
        };

        match result {
            Err(err) => {
                println!("Error while handling message:");
                println!("Msg: {:#?}", msg);
                println!("Error: {:#?}", err);
            },
            _ => {}
        }
    }
}
