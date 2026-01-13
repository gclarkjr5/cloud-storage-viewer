use crate::{app::Focus, components::connections::ConnectionComponentSelection, config::cloud_provider_config::cloud_provider_connection::CloudConnection};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    // ListConfiguration(Vec<u8>),
    ConnectionList(Vec<String>),
    // ViewerList(bool),
    Filter(Vec<String>),
    ChangeFocus(Focus),
    Nothing,
    Skip,
    // ActivateConfig(CloudProviderConfig),
    Activate(Vec<String>),
    SelectFilteredItem(String, Focus),
    Error(String),
}
