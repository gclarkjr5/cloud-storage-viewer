use std::io::BufRead;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GcloudConfig {
    name: String,
    is_active: String,
    account: String,
    project: String,
}

impl GcloudConfig {
    pub fn get_configs() -> Vec<Self> {
        Command::new("gcloud")
            .args(vec!["config", "configurations", "list"])
            .output()
            .expect("error getting config list")
            .stdout
            .lines()
            .skip(1)
            .map(|line| {
                let splits = line
                    .expect("error getting line in config list")
                    .split_whitespace()
                    .map(|split| split.to_string())
                    .collect::<Vec<String>>();

                Self {
                    name: splits[0].clone(),
                    is_active: splits[1].clone(),
                    account: splits[2].clone(),
                    project: splits[3].clone(),
                }
            })
            .collect::<Vec<Self>>()
    }

    pub fn get_active_config() -> Option<String> {
        let active_config = GcloudConfig::get_configs()
            .iter()
            .find(|config| config.is_active == "True")
            .expect("error finding active account")
            .name
            .clone();
        Some(active_config)
    }
}
