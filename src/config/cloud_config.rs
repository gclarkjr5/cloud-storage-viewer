use std::fmt::{self, Display, Error};
use std::io::BufRead;
use std::process::Command;
use std::result::Result;

use ego_tree::{NodeId, Tree};
use tracing::{error, info};

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
            info!("This is running somehow.");
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
    pub fn get_cloud_provider(&mut self, cloud_provider: String) -> Result<&mut CloudProvider, Action> {
        info!("Extracting Cloud Provider, {cloud_provider:?}, from Config");
        let found_cp = self.cloud_providers.iter_mut()
            .find(|cp| cp.to_string() == cloud_provider);

        if let Some(cp) = found_cp {
            info!("Cloud Provider retrieved: {cp:?}");
            Ok(cp)
        } else {
            Err(Action::Error("Could not find cloud provider".to_string()))
        }
    }

    pub fn verify_implemented_cloud_provider(
        &self,
        selection: String,
    ) -> Result<(), Action> {
        info!("Verifying tooling for {selection:?}");
        let cloud_provider = selection.into();
        match cloud_provider {
            CloudProvider::Azure(_) => {
                cloud_provider.check_cli_tools()?;
                info!("Tooling verified.");
                Ok(())
            }
            CloudProvider::Gcs(_) => {
                cloud_provider.check_cli_tools()?;
                info!("Tooling verified.");
                Ok(())
            }
            CloudProvider::S3(_) => {
                let message = format!("{} is not implemented yet", cloud_provider);
                error!("{message:?}");
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
        let cloud_provider_of_selection = path_identifier[1].clone();
        let cloud_provider = self.get_cloud_provider(cloud_provider_of_selection.clone()).expect("Error getting cp");

        let new_conn = path_identifier[2]
            .clone()
            .split('/')
            .last()
            .unwrap()
            .to_string();

        let current_conn = cloud_provider.get_active_config()?;
        if current_conn != new_conn {
            info!("Changing Connection from {current_conn:?} to {new_conn:?}");
            cloud_provider.activate_new_config(new_conn.clone())?;
        }
        self.update(cloud_provider_of_selection)?;
        info!("Connection Changed: {self:?}");

        Ok(Action::ActivateConfig(self.clone()))
    }

    fn update(&mut self, cloud_provider: String) -> Result<Action, Action> {
        // cloud_provider.update()?;
        let cp = self.get_cloud_provider(cloud_provider.to_string()).expect("Error getting cp");
        self.active_cloud_provider = Some(cp.clone());
        Ok(Action::Nothing)
    }

    pub fn list_config(&mut self, cloud_provider: CloudProvider) {
        self.cloud_providers
            .iter_mut()
            .for_each(|cp| match (cp.clone(), &cloud_provider) {
                (CloudProvider::Azure(_), CloudProvider::Azure(_)) => (),
                (CloudProvider::Gcs(_), CloudProvider::Gcs(_)) => (),
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
            Self::Azure(configs) => {
                configs.iter().for_each(|config| {
                    let res = config.name.to_string();

                    tree.get_mut(node_id)
                        .expect("error getting mutable node")
                        .append(res);
                });
                Ok(())
                
            },
            Self::Gcs(configs) => {
                configs.iter().for_each(|config| {
                    let res = config.name.to_string();

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
            Self::Azure(_) => {
                if Command::new("az").arg("--version").output().is_err() {
                    Err(Action::Error(
                        "Could not find requirement 'az'".to_string(),
                    ))
                } else {
                    Ok(())
                }
            },
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
    pub fn list_accounts(&mut self) -> Result<(), Action> {
        match self {
            Self::Azure(config) => {
                config.clear();
                let active_conn = match util::cli_command("az", &vec!["account", "show", "--query", "name", "--output", "yaml"]) {
                    Ok(data) => {
                        let name = data.lines().map(|line| line.expect("error getting azure listing")).collect::<Vec<String>>();
                        let needed_name = name[0].clone();
                        Ok(needed_name)
                    },
                    Err(e) => Err(e)
                }.expect("Error getting name out of Azure Active Conn");

                let cmd_args = vec!["account", "list", "--query", "[].name", "--output", "yaml"];
                let cmd_args_str = cmd_args.join(" ");

                info!("Listing Azure accounts via 'az {cmd_args_str:?}'");
                match Command::new("az")
                    .args(cmd_args)
                    .output()
                {
                    Ok(output) => {
                        info!("Successful listing.");
                        output.stdout.lines().for_each(|line| {
                            let splits = line
                                .expect("error getting line in config list")
                                .split_whitespace()
                                .map(|split| split.to_string())
                                .collect::<Vec<String>>();

                            let conf = AzureConfig {
                                name: splits[1].clone(),
                                is_active: splits[1] == active_conn,
                            };

                            config.push(conf);
                        });
                        Ok(())
                    }
                    Err(_) => Err(Action::Error(
                        "error calling gcloud config configurations list command".to_string(),
                    )),
                }
            },
            Self::Gcs(config) => {
                config.clear();

                let cmd_args = vec!["config", "configurations", "list"];
                let cmd_args_str = cmd_args.join(" ");
                info!("Listing GCP accounts via 'gcloud {cmd_args_str:?}'");

                match Command::new("gcloud")
                    .args(cmd_args)
                    .output()
                {
                    Ok(output) => {
                        info!("Successful listing.");
                        output.stdout.lines().skip(1).for_each(|line| {
                            let splits = line
                                .expect("error getting line in config list")
                                .split_whitespace()
                                .map(|split| split.to_string())
                                .collect::<Vec<String>>();

                            let conf = GcsConfig {
                                name: splits[0].clone(),
                                is_active: splits[1].to_lowercase().clone().parse().unwrap(),
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
        let name = self.to_string();
        info!("Retrieving active config from: {name:?}");

        match self {
            Self::Azure(confs) => {
                // run az account show and pull the name to get current default/active one
                // match util::cli_command("az", &vec!["account", "show", "--query", "name", "--output", "yaml"]) {
                //     Ok(data) => {
                //         let name = data.lines().map(|line| line.expect("error getting azure listing")).collect::<Vec<String>>();
                //         let needed_name = name[0].clone();
                //         info!("Active config: {needed_name:?}");
                //         Ok(needed_name.clone())
                //     },
                //     Err(e) => Err(e)
                // }
                let conf = confs.iter().find(|c| c.is_active);

                if let Some(az) = conf {
                    let needed_name = az.name.clone();
                    info!("Active config retrieved: {needed_name}");
                    Ok(needed_name)
                } else {
                    info!("NO ACTIVE AZURE CONF FOUND");
                    Ok(String::new())
                }
            }
            Self::Gcs(confs) => {
                let gcsconf = confs.iter().find(|c| c.is_active);

                if let Some(gc) = gcsconf {
                    let needed_name = gc.name.clone();
                    info!("Active config retrieved: {needed_name}");
                    Ok(needed_name)
                } else {
                    info!("NO ACTIVE GCS CONF FOUND");
                    Ok(String::new())
                }
            }
            Self::S3(_confs) => Err(Action::Error(
                "S3 get active config not implemented".to_string(),
            )),
        }
    }
    fn activate_new_config(&mut self, new_config: String) -> Result<(), Action> {
        match self {
            Self::Azure(_) => {
                let args = &vec!["account", "set", "--name", &new_config]; 
                info!("Activation command: az {args:?}");
                match util::cli_command(
                    "az",
                    args,
                ) {
                    Ok(_) => {
                        info!("Activation successful");

                        Ok(())
                    },
                    Err(e) => Err(e),
                }
                
            }
            Self::Gcs(confs) => {
                let args = &vec!["config", "configurations", "activate", &new_config]; 
                info!("Activation command: az {args:?}");
                match util::cli_command(
                    "gcloud",
                    args,
                ) {
                    Ok(_) => {
                        info!("Activation successful.");
                        confs.iter_mut().for_each(|conn| {
                            if conn.name == new_config {
                                conn.is_active = true;
                            } else {
                                conn.is_active = false;
                            }
                        });
                        info!("State edits successful.");
                        Ok(())
                        
                    },
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
            CloudProvider::Azure(_) => write!(f, "Azure Blob Storage"),
            CloudProvider::S3(_) => write!(f, "AWS S3"),
        }
    }
}

impl From<CloudProvider> for String {
    fn from(cloud_provider: CloudProvider) -> Self {
        match cloud_provider {
            CloudProvider::Azure(_) => "Azure Blob Storage".to_string(),
            CloudProvider::Gcs(_) => "Google Cloud Storage".to_string(),
            CloudProvider::S3(_) => "AWS S3".to_string(),
        }
    }
}

impl From<String> for CloudProvider {
    fn from(cloud_provider: String) -> Self {
        match cloud_provider.as_str() {
            "Azure Blob Storage" => CloudProvider::Azure(vec![AzureConfig::default()]),
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
    pub is_active: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GcsConfig {
    pub name: String,
    pub is_active: bool,
}
