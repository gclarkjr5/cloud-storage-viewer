use crate::{app::Focus};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    // ListConfiguration(Vec<u8>),
    ConnectionList(Vec<String>),
    ViewerList(Vec<String>),
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
