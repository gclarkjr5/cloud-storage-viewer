use std::fmt::{self, Display};
use std::io::BufRead;
use std::process::Command;
use std::result::Result;

use tracing::{error, info};

use crate::action::Action;
use crate::config::cloud_connection::{AzureConfig, CloudConnection, GcsConfig, S3Config};
use crate::config::cloud_provider_kind::CloudProviderKind;
use crate::util;

#[derive(Debug, Clone)]
pub struct CloudProviderConfig {
    pub s3: Vec<S3Config>,
    pub azure: Vec<AzureConfig>,
    pub gcs: Vec<GcsConfig>,
    pub active_cloud_connection: Option<CloudConnection>,
}

impl Display for CloudProviderConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(conn) = self.active_cloud_connection.clone() {
            let cloud: String = conn.clone().into();
            let name = conn.name();
            write!(f, "{name}({cloud})")
        // }
        // if self.active_cloud_connection.is_some() {

        //     let name = self.active_cloud_connection.unwrap().name();
        //     // let cloud: String = self.active_cloud_connection.unwrap().into();
        //     // let ac_res = self
        //     //     .active_cloud_connection
        //     //     .clone()
        //     //     .unwrap()
        //     //     .get_active_config();

        //     write!(f, "{name}")
        //     // if let Ok(ac) = ac_res {
        //     //     write!(f, "{ac}({cloud})")
        //     // } else {
        //     //     Err(Error)
        //     // }
        } else {
            write!(f, "No Active Cloud Provider")
        }
    }
}

impl Default for CloudProviderConfig {
    fn default() -> Self {
        Self {
            s3: vec![S3Config::default()],
            azure: vec![AzureConfig::default()],
            gcs: vec![GcsConfig::default()],
            active_cloud_connection: None,
        }
    }
}

impl CloudProviderConfig {
    pub fn all_cloud_providers(&self) -> &[CloudProviderKind] {
        &[
            CloudProviderKind::S3,
            CloudProviderKind::Azure,
            CloudProviderKind::Gcs,
        ]
        
    }

    pub fn list_connections(&mut self, cloud_provider_kind: &CloudProviderKind) -> Result<(), Action> {
        match cloud_provider_kind {
            CloudProviderKind::S3 => Err(Action::Error("Not implemented yet".to_string())),
            CloudProviderKind::Azure => {
                // clear out the vec
                self.azure.clear();

                // For Azure, you must run this command to see the active connection
                let active_conn = match util::cli_command("az", &vec!["account", "show", "--query", "name", "--output", "yaml"]) {
                    Ok(data) => {
                        let name = data.lines().map(|line| line.expect("error getting azure listing")).collect::<Vec<String>>();
                        let needed_name = name[0].clone();
                        Ok(needed_name)
                    },
                    Err(e) => Err(e)
                }.expect("Error getting name out of Azure Active Conn");

                // Now list out all available connections
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

                            self.azure.push(conf);
                        });
                        Ok(())
                    }
                    Err(_) => Err(Action::Error(
                        "error calling gcloud config configurations list command".to_string(),
                    )),
                }
                
            },
            CloudProviderKind::Gcs => {
                self.gcs.clear();

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

                            self.gcs.push(conf);
                        });
                        Ok(())
                    }
                    Err(_) => Err(Action::Error(
                        "error calling gcloud config configurations list command".to_string(),
                    )),
                }
                
            },
        }
    }

    pub fn activate(&mut self, cloud_provider_kind: &CloudProviderKind, account: Option<String>) -> Result<(), Action> {
        match account {
            None => {
                // No account means we just find the currently active account within
                // the cloud provider and make that the active connection
                match cloud_provider_kind {
                    CloudProviderKind::S3 => Err(Action::Error("".to_string())),
                    CloudProviderKind::Azure => {
                        let conf = self.azure.iter().find(|conf| conf.is_active);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Azure(c.clone()));
                        Ok(())
                    },
                    CloudProviderKind::Gcs => {
                        let conf = self.gcs.iter().find(|conf| conf.is_active);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Gcs(c.clone()));
                        Ok(())
                    },
                }
                
            }
            Some(acc) => {
                // An account means that it was selected by the user to change to
                match cloud_provider_kind {
                    CloudProviderKind::S3 => Err(Action::Error("".to_string())),
                    CloudProviderKind::Azure => {
                        let conf = self.azure.iter().find(|conf| conf.name == acc);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Azure(c.clone()));

                        self.azure.iter_mut().for_each(|conf| {
                            conf.is_active = conf.name == acc;
                        });
                        Ok(())
                    },
                    CloudProviderKind::Gcs => {
                        let conf = self.gcs.iter().find(|conf| conf.name == acc);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Gcs(c.clone()));

                        self.gcs.iter_mut().for_each(|conf| {
                            conf.is_active = conf.name == acc;
                        });
                        Ok(())
                    },
                }
                
            }
        }
    }

    pub fn verify_implemented_cloud_provider(
        &self,
        cloud_provider_kind: &CloudProviderKind,
    ) -> Result<(), Action> {
        info!("Verifying tooling for {cloud_provider_kind:?}");
        match cloud_provider_kind {
            CloudProviderKind::Azure => {
                cloud_provider_kind.check_cli_tools()?;
                info!("Tooling verified.");
                Ok(())
            }
            CloudProviderKind::Gcs => {
                cloud_provider_kind.check_cli_tools()?;
                info!("Tooling verified.");
                Ok(())
            }
            CloudProviderKind::S3 => {
                let message = format!("{} is not implemented yet", cloud_provider_kind);
                error!("{message:?}");
                Err(Action::Error(message))
            }
        }
    }

    // pub fn activate_config(&mut self, path_identifier: Vec<String>) -> Result<Action, Action> {
    //     // get the cloud in the path identifer as well as possible new connection
    //     let cloud_provider_of_selection = path_identifier[1].clone();
    //     let cloud_provider = self.get_cloud_provider(cloud_provider_of_selection.clone()).expect("Error getting cp");

    //     let new_conn = path_identifier[2]
    //         .clone()
    //         .split('/')
    //         .next_back()
    //         .unwrap()
    //         .to_string();

    //     let current_conn = cloud_provider.get_active_config()?;
    //     let current_conn_name = current_conn.name();
    //     if current_conn_name != new_conn {
    //         info!("Changing Connection from {current_conn_name:?} to {new_conn:?}");
    //         cloud_provider.activate_new_config(new_conn.clone())?;

    //         let new_cloud_conn = cloud_provider.get_active_config()?;
    //         self.active_cloud_connection = Some(new_cloud_conn);
    //         // self.set_active(cloud_provider_of_selection)?;
    //         info!("Connection Updated: {self:?}");

    //         Ok(Action::ActivateConfig(self.clone()))
    //     } else {
    //         Ok(Action::Nothing)
    //     }
    // }

    // fn set_active(&mut self, cloud_provider: String) -> Result<Action, Action> {
    //     info!("Setting Active Cloud Connection to {cloud_provider:?}");
    //     let cp = self.get_cloud_provider(cloud_provider.to_string()).expect("Error getting cp");
    //     self.active_cloud_connection = Some(cp.clone());
    //     Ok(Action::Nothing)
    // }

    // pub fn list_config(&mut self, cloud_provider: CloudProvider) {
    //     self.cloud_providers
    //         .iter_mut()
    //         .for_each(|cp| match (cp.clone(), &cloud_provider) {
    //             (CloudProvider::Azure(_), CloudProvider::Azure(_)) => (),
    //             (CloudProvider::Gcs(_), CloudProvider::Gcs(_)) => (),
    //             (CloudProvider::S3(_), CloudProvider::S3(_)) => (),
    //             _ => (),
    //         });
    //     // self.set_active_cloud(cloud_provider)
    // }
}

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum CloudProvider {
//     Azure(Vec<AzureConfig>),
//     Gcs(Vec<GcsConfig>),
//     S3(Vec<S3Config>),
// }

// impl Default for CloudProvider {
//     fn default() -> Self {
//         Self::Gcs(Vec::new())
//     }
// }

// impl CloudProvider {
    // pub fn create_nodes(&self, tree: &mut Tree<String>, node_id: NodeId) -> Result<(), Action> {
    //     match self {
    //         Self::Azure(configs) => {
    //             configs.iter().for_each(|config| {
    //                 let res = config.name.to_string();

    //                 tree.get_mut(node_id)
    //                     .expect("error getting mutable node")
    //                     .append(res);
    //             });
    //             Ok(())
                
    //         },
    //         Self::Gcs(configs) => {
    //             configs.iter().for_each(|config| {
    //                 let res = config.name.to_string();

    //                 tree.get_mut(node_id)
    //                     .expect("error getting mutable node")
    //                     .append(res);
    //             });
    //             Ok(())
    //         }
    //         Self::S3(_) => Err(Action::Error("S3 not implemented".to_string())),
    //     }
    // }
    // pub fn check_cli_tools(&self) -> Result<(), Action> {
    //     match self {
    //         Self::Azure(_) => {
    //             if Command::new("az").arg("--version").output().is_err() {
    //                 Err(Action::Error(
    //                     "Could not find requirement 'az'".to_string(),
    //                 ))
    //             } else {
    //                 Ok(())
    //             }
    //         },
    //         Self::Gcs(_) => {
    //             if Command::new("gcloud").arg("--version").output().is_err() {
    //                 Err(Action::Error(
    //                     "Could not find requirement 'gcloud'".to_string(),
    //                 ))
    //             } else if Command::new("gsutil").arg("--version").output().is_err() {
    //                 Err(Action::Error(
    //                     "Could not find requirement 'gsutil'".to_string(),
    //                 ))
    //             } else {
    //                 Ok(())
    //             }
    //         }
    //         Self::S3(_) => Err(Action::Error("S3 not implemented".to_string())),
    //     }
    // }
    // pub fn list_accounts(&mut self) -> Result<(), Action> {
    //     match self {
    //         Self::Azure(config) => {
    //             config.clear();
    //             let active_conn = match util::cli_command("az", &vec!["account", "show", "--query", "name", "--output", "yaml"]) {
    //                 Ok(data) => {
    //                     let name = data.lines().map(|line| line.expect("error getting azure listing")).collect::<Vec<String>>();
    //                     let needed_name = name[0].clone();
    //                     Ok(needed_name)
    //                 },
    //                 Err(e) => Err(e)
    //             }.expect("Error getting name out of Azure Active Conn");

    //             let cmd_args = vec!["account", "list", "--query", "[].name", "--output", "yaml"];
    //             let cmd_args_str = cmd_args.join(" ");

    //             info!("Listing Azure accounts via 'az {cmd_args_str:?}'");
    //             match Command::new("az")
    //                 .args(cmd_args)
    //                 .output()
    //             {
    //                 Ok(output) => {
    //                     info!("Successful listing.");
    //                     output.stdout.lines().for_each(|line| {
    //                         let splits = line
    //                             .expect("error getting line in config list")
    //                             .split_whitespace()
    //                             .map(|split| split.to_string())
    //                             .collect::<Vec<String>>();

    //                         let conf = AzureConfig {
    //                             name: splits[1].clone(),
    //                             is_active: splits[1] == active_conn,
    //                         };

    //                         config.push(conf);
    //                     });
    //                     Ok(())
    //                 }
    //                 Err(_) => Err(Action::Error(
    //                     "error calling gcloud config configurations list command".to_string(),
    //                 )),
    //             }
    //         },
    //         Self::Gcs(config) => {
    //             config.clear();

    //             let cmd_args = vec!["config", "configurations", "list"];
    //             let cmd_args_str = cmd_args.join(" ");
    //             info!("Listing GCP accounts via 'gcloud {cmd_args_str:?}'");

    //             match Command::new("gcloud")
    //                 .args(cmd_args)
    //                 .output()
    //             {
    //                 Ok(output) => {
    //                     info!("Successful listing.");
    //                     output.stdout.lines().skip(1).for_each(|line| {
    //                         let splits = line
    //                             .expect("error getting line in config list")
    //                             .split_whitespace()
    //                             .map(|split| split.to_string())
    //                             .collect::<Vec<String>>();

    //                         let conf = GcsConfig {
    //                             name: splits[0].clone(),
    //                             is_active: splits[1].to_lowercase().clone().parse().unwrap(),
    //                             // account: account.clone(),
    //                             // project: project.clone(),
    //                         };

    //                         config.push(conf);
    //                     });
    //                     Ok(())
    //                 }
    //                 Err(_) => Err(Action::Error(
    //                     "error calling gcloud config configurations list command".to_string(),
    //                 )),
    //             }
    //         }
    //         Self::S3(_) => Err(Action::Error("S3 update not implemented".to_string())),
    //     }
    // }
    // fn get_active_config(&self) -> Result<CloudConnection, Action> {
    //     let name = self.to_string();
    //     info!("Retrieving active config from: {name:?}");

    //     match self {
    //         Self::Azure(confs) => {
    //             let conf = confs.iter().find(|c| c.is_active);

    //             if let Some(conn) = conf {
    //                 // let needed_name = az.name.clone();
    //                 info!("Active config retrieved: {conn:?}");
    //                 Ok(CloudConnection::Azure(conn.clone()))
    //             } else {
    //                 info!("NO ACTIVE AZURE CONF FOUND");
    //                 Err(Action::Error("No active AZ config".to_string()))
    //                 // Ok(String::new())
    //             }
    //         }
    //         Self::Gcs(confs) => {
    //             let gcsconf = confs.iter().find(|c| c.is_active);

    //             if let Some(conn) = gcsconf {
    //                 // let needed_name = gc.name.clone();
    //                 info!("Active config retrieved: {conn:?}");
    //                 Ok(CloudConnection::Gcs(conn.clone()))
    //             } else {
    //                 info!("NO ACTIVE GCS CONF FOUND");
    //                 Err(Action::Error("No active GCS config".to_string()))
    //             }
    //         }
    //         Self::S3(_confs) => Err(Action::Error(
    //             "S3 get active config not implemented".to_string(),
    //         )),
    //     }
    // }
//     fn activate_new_config(&mut self, new_config: String) -> Result<(), Action> {
//         match self {
//             Self::Azure(confs) => {
//                 let args = &vec!["account", "set", "--name", &new_config]; 
//                 info!("Activation command: az {args:?}");
//                 match util::cli_command(
//                     "az",
//                     args,
//                 ) {
//                     Ok(_) => {
//                         info!("Activation successful");
//                         confs.iter_mut().for_each(|conn| {
//                             conn.is_active = conn.name == new_config
//                         });
//                         info!("State edits successful.");
//                         Ok(())
//                     },
//                     Err(e) => Err(e),
//                 }
                
//             }
//             Self::Gcs(confs) => {
//                 let args = &vec!["config", "configurations", "activate", &new_config]; 
//                 info!("Activation command: az {args:?}");
//                 match util::cli_command(
//                     "gcloud",
//                     args,
//                 ) {
//                     Ok(_) => {
//                         info!("Activation successful.");
//                         confs.iter_mut().for_each(|conn| {
//                             conn.is_active = conn.name == new_config
//                         });
//                         info!("State edits successful.");
//                         Ok(())
                        
//                     },
//                     Err(e) => Err(e),
//                 }
//             }
//             Self::S3(_confs) => unimplemented!(),
//         }
//     }
// }

// impl fmt::Display for CloudProvider {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             CloudProvider::Gcs(_) => write!(f, "Google Cloud Storage"),
//             CloudProvider::Azure(_) => write!(f, "Azure Blob Storage"),
//             CloudProvider::S3(_) => write!(f, "AWS S3"),
//         }
//     }
// }

// impl From<CloudProvider> for String {
//     fn from(cloud_provider: CloudProvider) -> Self {
//         match cloud_provider {
//             CloudProvider::Azure(_) => "Azure Blob Storage".to_string(),
//             CloudProvider::Gcs(_) => "Google Cloud Storage".to_string(),
//             CloudProvider::S3(_) => "AWS S3".to_string(),
//         }
//     }
// }

// impl From<String> for CloudProvider {
//     fn from(cloud_provider: String) -> Self {
//         match cloud_provider.as_str() {
//             "Azure Blob Storage" => CloudProvider::Azure(vec![AzureConfig::default()]),
//             "Google Cloud Storage" => CloudProvider::Gcs(vec![GcsConfig::default()]),
//             "AWS S3" => CloudProvider::S3(vec![S3Config::default()]),
//             _ => unimplemented!(),
//         }
//     }
// }


