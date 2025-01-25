use crate::{app::Focus, config::cloud_config::CloudConfig};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ListConfiguration(CloudConfig, Vec<String>, Vec<u8>),
    ListCloudProvider(CloudConfig),
    Filter(Vec<String>),
    ChangeFocus(Focus),
    Nothing,
    ActivateConfig(Vec<String>),
    SelectFilteredItem(String, Focus),
    Error(String),
}
