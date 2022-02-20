use crate::device::DeviceKey;

use super::{device::DeviceId, integration::IntegrationId};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible};

macro_attr! {
    #[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, NewtypeDisplay!)]
    pub struct GroupId(String);
}

impl GroupId {
    pub fn new(id: String) -> GroupId {
        GroupId(id)
    }
}

impl std::str::FromStr for GroupId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GroupId(s.to_string()))
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct GroupDeviceLink {
    pub integration_id: IntegrationId,
    pub device_id: Option<DeviceId>,
    pub name: Option<String>,
}

pub type GroupDeviceLinks = HashMap<GroupId, Vec<GroupDeviceLink>>;
pub type GroupDevicesConfig = Vec<GroupDeviceLink>;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GroupLink {
    pub group_id: GroupId,
}

pub type GroupLinksConfig = Vec<GroupLink>;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GroupConfig {
    pub name: String,
    pub devices: Option<GroupDevicesConfig>,
    pub groups: Option<GroupLinksConfig>,
}

pub type GroupsConfig = HashMap<GroupId, GroupConfig>;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct FlattenedGroupConfig {
    pub name: String,
    pub device_ids: Vec<DeviceKey>,
}

pub type FlattenedGroupsConfig = HashMap<GroupId, FlattenedGroupConfig>;
