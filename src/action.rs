use crate::config::cloud_config::CloudConfig;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ListConfiguration(CloudConfig, Vec<String>, Vec<u8>),
    ListCloudProvider(CloudConfig),
    // NextPage,
    // PreviousPage,
    ChangeFocus,
    Nothing,
    ActivateConfig(Vec<String>),
}
