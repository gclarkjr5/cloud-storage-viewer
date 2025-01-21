use std::fmt::{self, Display};
use std::io::{BufRead, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CloudConfig {
    pub cloud_providers: Vec<CloudProvider>,
    pub active_cloud_provider: Option<CloudProvider>,
}

impl Display for CloudConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.active_cloud_provider.is_some() {
            let cloud: String = self.active_cloud_provider.clone().unwrap().into();
            let ac = self
                .active_cloud_provider
                .clone()
                .unwrap()
                .get_active_config();

            write!(f, "{ac}({cloud})")
        } else {
            write!(f, "No Active Cloud Provider")
        }
    }
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            cloud_providers: vec![
                CloudProvider::Gcs(Vec::new()),
                CloudProvider::Azure(Vec::new()),
                CloudProvider::S3(Vec::new()),
            ],
            active_cloud_provider: None,
        }
    }
}

impl CloudConfig {
    pub fn init(&mut self) -> Result<()> {
        for cloud_provider in self.cloud_providers.iter_mut() {
            cloud_provider.init()
        }
        Ok(())
    }

    pub fn set_active_cloud(&mut self, cloud_provider: CloudProvider) {
        self.cloud_providers
            .iter()
            .for_each(|cp| match (cp, cloud_provider.clone()) {
                (CloudProvider::Azure(_), CloudProvider::Azure(_)) => {
                    self.active_cloud_provider = Some(cp.clone())
                }
                (CloudProvider::Gcs(_), CloudProvider::Gcs(_)) => {
                    self.active_cloud_provider = Some(cp.clone())
                }
                (CloudProvider::S3(_), CloudProvider::S3(_)) => {
                    self.active_cloud_provider = Some(cp.clone())
                }
                _ => (),
            });
    }

    pub fn activate_config(&mut self, path_identifier: Vec<String>) -> Result<()> {
        // get the cloud in the path identifer as well as possible new connection
        let cloud_provider = path_identifier[1].clone().into();
        let new_config = path_identifier[2]
            .clone()
            .split('/')
            .last()
            .unwrap()
            .to_string();

        self.cloud_providers
            .iter()
            .for_each(|cp| match (cp, &cloud_provider) {
                (CloudProvider::Azure(_), CloudProvider::Azure(_)) => (),
                (CloudProvider::Gcs(_), CloudProvider::Gcs(_)) => {
                    let current_conn = cp.get_active_config();

                    if current_conn != new_config {
                        cp.activate_new_config(new_config.clone());
                    }
                }
                (CloudProvider::S3(_), CloudProvider::S3(_)) => (),
                _ => (),
            });
        // self.init();
        self.update(cloud_provider);

        Ok(())
    }

    fn update(&mut self, cloud_provider: CloudProvider) {
        self.list_config(cloud_provider.clone());
        self.set_active_cloud(cloud_provider)
    }

    pub fn list_config(&mut self, cloud_provider: CloudProvider) {
        self.cloud_providers
            .iter_mut()
            .for_each(|cp| match (cp.clone(), &cloud_provider) {
                (CloudProvider::Azure(_), CloudProvider::Azure(_)) => (),
                (CloudProvider::Gcs(_), CloudProvider::Gcs(_)) => cp.init(),
                (CloudProvider::S3(_), CloudProvider::S3(_)) => (),
                _ => (),
            });
        self.set_active_cloud(cloud_provider)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudProvider {
    Azure(Vec<AzureConfig>),
    Gcs(Vec<GcsConfig>),
    S3(Vec<S3Config>),
}

impl Default for CloudProvider {
    fn default() -> Self {
        Self::Gcs(Vec::new())
    }
}

trait CloudProviderInit {
    fn init(&mut self);
    fn get_active_config(&self) -> String;
    fn activate_new_config(&self, new_connection: String);
}

impl CloudProviderInit for CloudProvider {
    fn init(&mut self) {
        match self {
            Self::Azure(_) => (),
            Self::Gcs(config) => {
                config.clear();
                Command::new("gcloud")
                    .args(vec!["config", "configurations", "list"])
                    .output()
                    .expect("error getting config list")
                    .stdout
                    .lines()
                    .skip(1)
                    .for_each(|line| {
                        let splits = line
                            .expect("error getting line in config list")
                            .split_whitespace()
                            .map(|split| split.to_string())
                            .collect::<Vec<String>>();

                        let conf = GcsConfig {
                            name: splits[0].clone(),
                            is_active: splits[1].clone(),
                            // account: account.clone(),
                            // project: project.clone(),
                        };

                        config.push(conf);
                    });
            }
            Self::S3(_) => (),
        }
    }
    fn get_active_config(&self) -> String {
        match self {
            Self::Azure(_confs) => unimplemented!(),
            Self::Gcs(confs) => {
                let gcsconf = confs.iter().find(|c| c.is_active == "True");

                if let Some(gc) = gcsconf {
                    gc.name.clone()
                } else {
                    String::new()
                }
            }
            Self::S3(_confs) => unimplemented!(),
        }
    }
    fn activate_new_config(&self, new_config: String) {
        match self {
            Self::Azure(_confs) => unimplemented!(),
            Self::Gcs(_) => {
                Command::new("gcloud")
                    .args(["config", "configurations", "activate"])
                    .arg(new_config)
                    .output()
                    .expect("error creating new gcloud config");
            }
            Self::S3(_confs) => unimplemented!(),
        }
    }
}

impl fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CloudProvider::Gcs(_) => write!(f, "Google Cloud Storage"),
            CloudProvider::Azure(_) => write!(f, "Azure Data Lake Storage"),
            CloudProvider::S3(_) => write!(f, "AWS S3"),
        }
    }
}

impl From<CloudProvider> for String {
    fn from(cloud_provider: CloudProvider) -> Self {
        match cloud_provider {
            CloudProvider::Azure(_) => "Azure Data Lake Storage".to_string(),
            CloudProvider::Gcs(_) => "Google Cloud Storage".to_string(),
            CloudProvider::S3(_) => "AWS S3".to_string(),
        }
    }
}

impl From<String> for CloudProvider {
    fn from(cloud_provider: String) -> Self {
        match cloud_provider.as_str() {
            "Azure Data Lake Storage" => CloudProvider::Azure(vec![AzureConfig::default()]),
            "Google Cloud Storage" => CloudProvider::Gcs(vec![GcsConfig::default()]),
            "AWS S3" => CloudProvider::S3(vec![S3Config::default()]),
            _ => unimplemented!(),
        }
    }
}

// impl Cloud {
//     fn init(&mut self) -> Result<()> {
//         match self {
//             // Self::Azure(config) => config.init()?,
//             Self::Gcs(config) => config.init(),
//             // Self::S3(config) => config.init()?,
//             _ => Ok(()),
//         }
//     }
// }

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3Config {
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AzureConfig {
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GcsConfig {
    pub name: String,
    pub is_active: String,
}

// impl Conf for GcsConfig {}
// impl GcsConfig {
//     fn default() -> Box<dyn Conf> {
//         Box::new(GcsConfig {
//             name: String::new(),
//             is_active: String::new(),
//         })
//     }
// }
// impl Conf for S3Config {}
// impl Conf for AzureConfig {}

// impl CreateConfig<S3Config> for Cloud {
//     fn list() -> Self {
//         Self::S3(vec![S3Config::default()])
//     }
// }

// impl CreateConfig<AzureConfig> for Cloud {
//     fn list() -> Self {
//         Self::Azure(vec![AzureConfig::default()])
//     }
// }
