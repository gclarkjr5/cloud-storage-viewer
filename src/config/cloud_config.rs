use std::fmt::{self, Display, Error};
use std::io::BufRead;
use std::process::Command;
use std::result::Result;

use ego_tree::{NodeId, Tree};

use crate::action::Action;
use crate::util;

#[derive(Debug, Clone)]
pub struct CloudConfig {
    pub cloud_providers: Vec<CloudProvider>,
    pub active_cloud_provider: Option<CloudProvider>,
}

impl Display for CloudConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.active_cloud_provider.is_some() {
            let cloud: String = self.active_cloud_provider.clone().unwrap().into();
            let ac_res = self
                .active_cloud_provider
                .clone()
                .unwrap()
                .get_active_config();

            if let Ok(ac) = ac_res {
                write!(f, "{ac}({cloud})")
            } else {
                Err(Error)
            }
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
    pub fn verify_implemented_cloud_provider(
        &self,
        cloud_provider: CloudProvider,
    ) -> Result<CloudProvider, Action> {
        match cloud_provider {
            CloudProvider::Azure(_) => {
                let message = format!("{} is not implemented yet", cloud_provider);
                Err(Action::Error(message))
            }
            CloudProvider::Gcs(_) => {
                cloud_provider.check_cli_tools()?;
                Ok(cloud_provider)
            }
            CloudProvider::S3(_) => {
                let message = format!("{} is not implemented yet", cloud_provider);
                Err(Action::Error(message))
            }
        }
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

    pub fn activate_config(&mut self, path_identifier: Vec<String>) -> Result<Action, Action> {
        // get the cloud in the path identifer as well as possible new connection
        let mut cloud_provider: CloudProvider = path_identifier[1].clone().into();
        let new_config = path_identifier[2]
            .clone()
            .split('/')
            .last()
            .unwrap()
            .to_string();

        let current_conn = cloud_provider.get_active_config()?;
        if current_conn != new_config {
            cloud_provider.activate_new_config(new_config.clone())?;
        }

        self.update(&mut cloud_provider)?;
        Ok(Action::ActivateConfig(self.clone()))
    }

    fn update(&mut self, cloud_provider: &mut CloudProvider) -> Result<Action, Action> {
        cloud_provider.update()?;
        self.active_cloud_provider = Some(cloud_provider.clone());
        Ok(Action::Nothing)
    }

    pub fn list_config(&mut self, cloud_provider: CloudProvider) {
        self.cloud_providers
            .iter_mut()
            .for_each(|cp| match (cp.clone(), &cloud_provider) {
                (CloudProvider::Azure(_), CloudProvider::Azure(_)) => (),
                (CloudProvider::Gcs(_), CloudProvider::Gcs(_)) => cp.update().unwrap(),
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

impl CloudProvider {
    pub fn create_nodes(&self, tree: &mut Tree<String>, node_id: NodeId) -> Result<(), Action> {
        match self {
            Self::Azure(_) => Err(Action::Error("Azure not implemented".to_string())),
            Self::Gcs(configs) => {
                configs.iter().for_each(|config| {
                    let res = format!("{}/{}", self, config.name.clone());

                    tree.get_mut(node_id)
                        .expect("error getting mutable node")
                        .append(res);
                });
                Ok(())
            }
            Self::S3(_) => Err(Action::Error("S3 not implemented".to_string())),
        }
    }
    pub fn check_cli_tools(&self) -> Result<(), Action> {
        match self {
            Self::Azure(_) => Err(Action::Error("Azure not implemented".to_string())),
            Self::Gcs(_) => {
                if Command::new("gcloud").arg("--version").output().is_err() {
                    Err(Action::Error(
                        "Could not find requirement 'gcloud'".to_string(),
                    ))
                } else if Command::new("gsutil").arg("--version").output().is_err() {
                    Err(Action::Error(
                        "Could not find requirement 'gsutil'".to_string(),
                    ))
                } else {
                    Ok(())
                }
            }
            Self::S3(_) => Err(Action::Error("S3 not implemented".to_string())),
        }
    }
    pub fn update(&mut self) -> Result<(), Action> {
        match self {
            Self::Azure(_) => Err(Action::Error("Azure update not implemented".to_string())),
            Self::Gcs(config) => {
                config.clear();

                match Command::new("gcloud")
                    .args(vec!["config", "configurations", "list"])
                    .output()
                {
                    Ok(output) => {
                        output.stdout.lines().skip(1).for_each(|line| {
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
                        Ok(())
                    }
                    Err(_) => Err(Action::Error(
                        "error calling gcloud config configurations list command".to_string(),
                    )),
                }
            }
            Self::S3(_) => Err(Action::Error("S3 update not implemented".to_string())),
        }
    }
    fn get_active_config(&self) -> Result<String, Action> {
        match self {
            Self::Azure(_confs) => Err(Action::Error(
                "Azure get active config not implemented".to_string(),
            )),
            Self::Gcs(confs) => {
                let gcsconf = confs.iter().find(|c| c.is_active == "True");

                if let Some(gc) = gcsconf {
                    Ok(gc.name.clone())
                } else {
                    Ok(String::new())
                }
            }
            Self::S3(_confs) => Err(Action::Error(
                "S3 get active config not implemented".to_string(),
            )),
        }
    }
    fn activate_new_config(&self, new_config: String) -> Result<(), Action> {
        match self {
            Self::Azure(_confs) => unimplemented!(),
            Self::Gcs(_) => {
                match util::cli_command(
                    "gcloud",
                    &vec!["config", "configurations", "activate", &new_config],
                ) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
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
