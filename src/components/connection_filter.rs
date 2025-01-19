use super::Component;
use crate::config::Config;

#[derive(Debug, Default)]
pub struct ConnectionFilter {
    pub config: Config,
    pub active: bool,
}

// impl Component for ConnectionFilter {}
