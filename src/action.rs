use crate::{app::Focus, config::cloud_provider_config::CloudProviderConfig};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ListConfiguration(CloudProviderConfig, Vec<u8>),
    ListCloudProvider(CloudProviderConfig),
    Filter(Vec<String>),
    ChangeFocus(Focus),
    Nothing,
    Skip,
    ActivateConfig(CloudProviderConfig),
    SelectFilteredItem(String, Focus),
    Error(String),
}
