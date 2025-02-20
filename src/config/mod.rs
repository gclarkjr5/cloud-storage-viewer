use cloud_config::CloudConfig;
use key_config::KeyConfig;

pub mod cloud_config;
pub mod key_config;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub key_config: KeyConfig,
    pub cloud_config: CloudConfig,
}
