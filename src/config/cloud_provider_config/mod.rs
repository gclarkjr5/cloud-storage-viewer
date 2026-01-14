use std::fmt::{self, Display};
use std::io::BufRead;
use std::process::Command;
use std::result::Result;

pub mod cloud_provider_kind;
pub mod cloud_provider_connection; 

use tracing::{error, info};

use crate::action::Action;
use crate::app::Focus;
use crate::components::connections::ConnectionComponentSelection;
// use crate::components::connections::ListingRequest;
use cloud_provider_connection::{AzureConfig, CloudConnection, GcsConfig, S3Config};
use cloud_provider_kind::CloudProviderKind;
use crate::util;

#[derive(Debug, Clone, Default)]
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
        } else {
            write!(f, "No Active Cloud Provider")
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
                let mut active_conn = String::new();

                if let Ok(data) = util::cli_command("az", &vec!["account", "show", "--query", "name", "--output", "yaml"]) {
                    match data.lines().next() {
                        None => (),
                        Some(ln) => match ln {
                            Err(_) => (),
                            Ok(l) => active_conn = l
                        }
                    }
                }

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
                            match line {
                                Err(_) => (),
                                Ok(ln) => {
                                    match ln.split_whitespace().last() {
                                        None => (),
                                        Some(l) => {
                                            let conf = AzureConfig { name: l.to_string(), is_active: l == active_conn };
                                            if conf.is_active {
                                                self.active_cloud_connection = Some(CloudConnection::Azure(conf.clone()));
                                            }
                                            self.azure.push(conf)
                                        }
                                    }
                                }
                            }
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
                            match line {
                                Err(_) => (),
                                Ok(ln) => {
                                    let mut lsplit = ln.split_whitespace();
                                    let name = match lsplit.next() {
                                        None => String::new(),
                                        Some(l) => l.to_string()
                                    };
                                    let is_active = match lsplit.next() {
                                        None => false,
                                        Some(l) => {
                                            let lowered = l.to_lowercase();
                                            lowered.parse::<bool>().unwrap_or_default()

                                        }
                                    };
                                    let conf = GcsConfig { name: name.to_string(), is_active, data: None };
                                    if conf.is_active {
                                        self.active_cloud_connection = Some(CloudConnection::Gcs(conf.clone()));
                                    }
                                    self.gcs.push(conf);
                                }
                            }
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

    // pub fn activate(&mut self, cloud_provider_kind: &CloudProviderKind, account: Option<String>) -> Result<(), Action> {
    pub fn activate(&mut self, selection: Vec<String>) -> Result<(), Action> {
        if selection.len() < 2 {
            return Err(Action::Error("Cannot Activate Connections".to_string()))
        }

        let sel: ConnectionComponentSelection = selection.into();
        match sel.cloud_provider_connection {
            None => {
                // No account means we just find the currently active account within
                // the cloud provider and make that the active connection
                match sel.cloud_provider_kind {
                    CloudProviderKind::S3 => Err(Action::Error("Not Implemented".to_string())),
                    CloudProviderKind::Azure => {
                        if self.azure.is_empty() {
                            return Err(Action::Error("Cannot Activate a Cloud Provider before listing one. Please list it by pressing [Enter].".to_string()))
                        }
                        let conf = self.azure.iter().find(|conf| conf.is_active);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Azure(c.clone()));
                        info!("Active Cloud Connection: {:?}", self.active_cloud_connection);
                        Ok(())
                    },
                    CloudProviderKind::Gcs => {
                        if self.gcs.is_empty() {
                            return Err(Action::Error("Cannot Activate a Cloud Provider before listing one. Please list it by pressing [Enter].".to_string()))
                        }
                        let conf = self.gcs.iter().find(|conf| conf.is_active);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Gcs(c.clone()));
                        info!("Active Cloud Connection: {:?}", self.active_cloud_connection);
                        Ok(())
                    },
                }
                
            }
            Some(acc) => {
                // An account means that it was selected by the user to change to
                match sel.cloud_provider_kind {
                    CloudProviderKind::S3 => Err(Action::Error("".to_string())),
                    CloudProviderKind::Azure => {
                        // mutate all to change to requested account
                        self.azure.iter_mut().for_each(|conf| {
                            conf.is_active = conf.name == acc;
                        });

                        let conf = self.azure.iter().find(|conf| conf.is_active);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Azure(c.clone()));

                        info!("Active Cloud Connection: {:?}", self.active_cloud_connection);
                        Ok(())
                    },
                    CloudProviderKind::Gcs => {
                        // mutate all to change to requested account
                        self.gcs.iter_mut().for_each(|conf| {
                            conf.is_active = conf.name == acc;
                        });

                        // return newly active and set as active
                        let conf = self.gcs.iter().find(|conf| conf.is_active);
                        self.active_cloud_connection = conf.map(|c| CloudConnection::Gcs(c.clone()));

                        info!("Active Cloud Connection: {:?}", self.active_cloud_connection);
                        Ok(())
                    },
                }
                
            }
        }
    }

    // pub fn verify_implemented_cloud_provider(
    //     &self,
    //     cloud_provider_kind: &CloudProviderKind,
    // ) -> Result<(), Action> {
    //     info!("Verifying tooling for {cloud_provider_kind:?}");
    //     match cloud_provider_kind {
    //         CloudProviderKind::Azure => {
    //             cloud_provider_kind.check_cli_tools()?;
    //             info!("Tooling verified.");
    //             Ok(())
    //         }
    //         CloudProviderKind::Gcs => {
    //             cloud_provider_kind.check_cli_tools()?;
    //             info!("Tooling verified.");
    //             Ok(())
    //         }
    //         CloudProviderKind::S3 => {
    //             let message = format!("{} is not implemented yet", cloud_provider_kind);
    //             error!("{message:?}");
    //             Err(Action::Error(message))
    //         }
    //     }
    // }

    pub fn ls(&mut self, selection: Vec<String>) -> Result<Focus, Action> {
        if selection.len() < 2 {
            return Err(Action::Error("Cannot List Connections".to_string()))
        }

        let connection_selection: ConnectionComponentSelection = selection.clone().into();
        match connection_selection.cloud_provider_connection {
            None => {
                // No account means we just re-list the Cloud Provider
                self.list_connections(&connection_selection.cloud_provider_kind)?;
                self.activate(selection)?;
                Ok(Focus::Connections)
            }
            Some(conn) => {
                self.activate(selection)?;
                match &self.active_cloud_connection {
                    None => Ok(Focus::Connections),
                    Some(cloud_connection) => {
                        match cloud_connection {
                            CloudConnection::S3(_conf) => Err(Action::Error("Not implemented yet".to_string())),
                            CloudConnection::Azure(_conf) =>{
                                // conf.ls();
                                Err(Action::Error("No Azure yet".to_string()))
                            }
                            CloudConnection::Gcs(conf) => {
                                let output = conf.ls();
                                self.active_cloud_connection = match output {
                                    Err(_e) => None,
                                    Ok(out) => Some(CloudConnection::Gcs(
                                        GcsConfig { name: conf.name.clone(), is_active: true, data: Some(out) }
                                    ))
                                };
                                Ok(Focus::Viewer)
                                // Ok(Action::Nothing)
                        
                            }
                        }
                    }
                }
                // Ok(Action::Nothing)
            }
        }
        // this scenario is only for listing connections of a cloud provider
        // if connection_selection.cloud_provider_connection.is_none() {
        //     // return Ok(Action::ActivateConfig(self.clone()))
        // } else {
        //     Ok(Action::Error("Cannot do connections yet".to_string()))
        // }
        // activate the account requested
        // self.activate(&listing_request.provider_kind, listing_request.connection.clone())?;

        // do the listing
    }
}
