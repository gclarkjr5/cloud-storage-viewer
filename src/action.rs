use crate::{
    app::Focus,
    config::cloud_config::{CloudConfig, CloudProvider},
};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ListConfiguration(CloudConfig, Vec<String>, Vec<u8>),
    ListCloudProvider(CloudConfig),
    // ListCloudProvider(CloudProvider),
    Filter(Vec<String>),
    ChangeFocus(Focus),
    Nothing,
    Skip,
    ActivateConfig(Vec<String>),
    SelectFilteredItem(String, Focus),
    Error(String),
}
