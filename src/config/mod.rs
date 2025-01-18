use cloud_config::CloudConfig;
use key_config::KeyConfig;

use std::io::Result;
pub mod cloud_config;
pub mod key_config;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub key_config: KeyConfig,
    pub cloud_config: CloudConfig,
}

impl Config {
    pub fn init(&mut self) -> Result<()> {
        // self.key_config.init()?;
        self.cloud_config.init()?;
        Ok(())
    }
}
