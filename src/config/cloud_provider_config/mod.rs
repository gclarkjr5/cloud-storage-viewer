use std::fmt::{self, Display};
use std::io::BufRead;
use std::process::Command;
use std::result::Result;

pub mod cloud_provider_kind;
pub mod cloud_provider_connection; 

use tracing::{error, info};

use crate::action::Action;
use cloud_provider_connection::{AzureConfig, CloudConnection, GcsConfig, S3Config};
use cloud_provider_kind::CloudProviderKind;
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
                                            self.azure.push(
                                                AzureConfig { name: l.to_string(), is_active: l == active_conn }
                                            )
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

                                    self.gcs.push(
                                        GcsConfig { name, is_active }
                                    );
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

    pub fn ls(&mut self, cloud_provider_kind: &CloudProviderKind, account: Option<String>) -> Result<Action, Action> {
        // an account must exist in the selection
        if account.is_none() {
            return Err(Action::Error("Only Connections/Accounts can be listed".to_string()))
        }
        // activate the account requested
        self.activate(cloud_provider_kind, account)?;

        // do the listing
        match &self.active_cloud_connection {
            None => Ok(Action::Nothing),
            Some(cloud_connection) => {
                match cloud_connection {
                    CloudConnection::S3(_conf) => Err(Action::Error("Not implemented yet".to_string())),
                    CloudConnection::Azure(_conf) =>{
                        // conf.ls();
                        Ok(Action::Nothing)
                    }
                    CloudConnection::Gcs(_conf) => {
                        // conf.ls();
                        Ok(Action::Nothing)
                        
                    }
                }
            }
        }
    }
}
