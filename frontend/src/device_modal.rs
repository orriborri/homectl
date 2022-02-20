use dioxus::{events::FormEvent, prelude::*};
use dioxus_websocket_hooks::use_ws_context;
use homectl_types::{device::Device, event::Message, websockets::WebSocketRequest};

use crate::{
    color_swatch::ColorSwatch,
    modal::Modal,
    util::{hsv_to_css_hsl_str, scale_hsv_value_to_display},
};

#[derive(Props)]
pub struct DeviceModalProps<'a> {
    device: &'a Device,
    modal_open: &'a bool,
    set_modal_open: &'a UseState<bool>,
}

#[allow(non_snake_case)]
pub fn DeviceModal<'a>(cx: Scope<'a, DeviceModalProps<'a>>) -> Element<'a> {
    let ws = use_ws_context(&cx);

    let (show_debug, _set_show_debug) = use_state(&cx, || false);
    // let toggle_debug = move |_: MouseEvent| {
    //     let mut show_debug = show_debug.modify();
    //     *show_debug = !*show_debug;
    // };

    let show_hsv = match &cx.props.device.state{
        homectl_types::device::DeviceState::Light(l) => match &l.capabilities {
            Some(c) => c.Hsv,
            None => false,
        },
        _ => false
    };
    let show_cct = match &cx.props.device.state{
        homectl_types::device::DeviceState::Light(l) => match &l.capabilities {
            Some(c) => c.Cct,
            None => false,
        },
        _ => false
    };
    let (show_brightness, set_show_brightness) = use_state(&cx, || show_hsv || show_cct);

    let brightness = cx.props.device.state.get_brightness().unwrap_or_default();
    let power = cx.props.device.state.is_powered_on().unwrap_or_default();
    let color = cx.props.device.state.get_color();

    let hue = color.unwrap_or_default().hue.to_positive_degrees();
    let saturation = color.unwrap_or_default().saturation;
    let cct = cx
        .props
        .device
        .state
        .get_cct()
        .unwrap_or_default()
        .get_cct();

    let sat_min = {
        let mut color = color.unwrap_or_default();
        color.value = 1.0;
        color.saturation = 0.0;
        hsv_to_css_hsl_str(&Some(color))
    };

    let sat_max = {
        let mut color = color.unwrap_or_default();
        color.value = 1.0;
        color.saturation = 1.0;
        hsv_to_css_hsl_str(&Some(color))
    };

    let bri_min = {
        let mut color = color.unwrap_or_default();
        color.value = 0.5;
        hsv_to_css_hsl_str(&Some(color))
    };

    let bri_max = {
        let mut color = color.unwrap_or_default();
        color.value = 1.0;
        hsv_to_css_hsl_str(&Some(color))
    };

    let set_power = {
        let ws = ws.clone();
        move |evt: FormEvent| {
            let power: Option<bool> = evt.data.value.parse().ok();

            if let Some(power) = power {
                let mut device = cx.props.device.clone();
                device.state.set_power(power);
                device.scene = None;
                ws.send_json(&WebSocketRequest::Message(Message::SetDeviceState {
                    device,
                    set_scene: true,
                }))
            }
        }
    };

    let set_hue = {
        let ws = ws.clone();
        move |evt: FormEvent| {
            let hue: Option<f32> = evt.data.value.parse().ok();

            if let Some(hue) = hue {
                let mut device = cx.props.device.clone();
                device.state.set_hue(hue);
                device.scene = None;
                ws.send_json(&WebSocketRequest::Message(Message::SetDeviceState {
                    device,
                    set_scene: true,
                }))
            }
        }
    };

    let set_saturation = {
        let ws = ws.clone();
        move |evt: FormEvent| {
            let saturation: Option<f32> = evt.data.value.parse().ok();

            if let Some(saturation) = saturation {
                let mut device = cx.props.device.clone();
                device.state.set_saturation(saturation);
                device.scene = None;
                ws.send_json(&WebSocketRequest::Message(Message::SetDeviceState {
                    device,
                    set_scene: true,
                }))
            }
        }
    };

    let set_brightness = {
        let ws = ws.clone();
        move |evt: FormEvent| {
            let value: Option<f32> = evt.data.value.parse().ok();

            if let Some(value) = value {
                let mut device = cx.props.device.clone();
                device.state.set_brightness(value);
                device.scene = None;
                ws.send_json(&WebSocketRequest::Message(Message::SetDeviceState {
                    device,
                    set_scene: true,
                }))
            }
        }
    };

    let set_cct = move |evt: FormEvent| {
        let cct: Option<f32> = evt.data.value.parse().ok();

        if let Some(cct) = cct {
            let mut device = cx.props.device.clone();
            device.state.set_cct(cct);
            device.scene = None;
            ws.send_json(&WebSocketRequest::Message(Message::SetDeviceState {
                device,
                set_scene: true,
            }))
        }
    };

    let device_debug = format!("{:#?}", cx.props.device);

    cx.render(rsx! {
        Modal {
            title: "{cx.props.device.name}",
            modal_open: cx.props.modal_open,
            set_modal_open: cx.props.set_modal_open,
            contents: cx.render(rsx! {
                div {
                    class: "gap-4, flex flex-col flex-1",

                    "Power on:"
                    input {
                        r#type: "checkbox",
                        checked: "{power}",
                        onchange: set_power
                    }

                    "Color:",
                    ColorSwatch { color: color.map(scale_hsv_value_to_display) },
                    show_hsv.then(||rsx! {
                    "Hue:",
                    style {
                        ".hue-slider::-webkit-slider-runnable-track {{
                            background: linear-gradient(to right, #ff0000 0%, #ffff00 17%, #00ff00 33%, #00ffff 50%, #0000ff 67%, #ff00ff 83%, #ff0000 100%);
                            border-radius: 0.5rem;
                            height: 1rem;
                            border: 1px solid #cccccc;
                        }}"
                    }
                    input {
                        class: "hue-slider",
                        r#type: "range",
                        min: "0",
                        max: "359",
                        value: "{hue}",
                        onchange: set_hue
                    }

                    "Saturation:",
                    style {
                        ".saturation-slider::-webkit-slider-runnable-track {{
                            background: linear-gradient(to right, {sat_min} 0%, {sat_max} 100%);
                            border-radius: 0.5rem;
                            height: 1rem;
                            border: 1px solid #cccccc;
                        }}"
                    }
                    input {
                        class: "saturation-slider",
                        r#type: "range",
                        min: "0",
                        max: "1",
                        step: "0.01",
                        value: "{saturation}",
                        onchange: set_saturation
                    }
                    })
                    show_brightness.then(||rsx!{
                    "Brightness:",
                    style {
                        ".brightness-slider::-webkit-slider-runnable-track {{
                            background: linear-gradient(to right, {bri_min} 0%, {bri_max} 100%);
                            border-radius: 0.5rem;
                            height: 1rem;
                            border: 1px solid #cccccc;
                        }}"
                    }
                    input {
                        class: "brightness-slider",
                        r#type: "range",
                        min: "0",
                        max: "1",
                        step: "0.01",
                        value: "{brightness}",
                        onchange: set_brightness
                    }
                })
                show_cct.then(||rsx!{
                    "Color temperature:",
                    style {
                        ".cct-slider::-webkit-slider-runnable-track {{
                            background: linear-gradient(to right, #ffbb7b 0%, #ffffff 50%, #9db4ff 100%);
                            border-radius: 0.5rem;
                            height: 1rem;
                            border: 1px solid #cccccc;
                        }}"
                    }
                    input {
                        class: "cct-slider",
                        r#type: "range",
                        min: "2000",
                        max: "6500",
                        step: "1",
                        value: "{cct}",
                        onchange: set_cct
                    }
                
                })
                show_debug.then(|| rsx! {
                    div {
                        class: "flex-1 overflow-auto min-h-[300px]",

                        pre {
                            class: "m-0",

                            "{device_debug}"
                        }
                    }
                })
            }
        })
    }
})}
