use std::{fmt, process::Command};

use crate::action::Action;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudProviderKind {
    S3,
    Azure,
    Gcs,
}

impl fmt::Display for CloudProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CloudProviderKind::Gcs => write!(f, "Google Cloud Storage"),
            CloudProviderKind::Azure => write!(f, "Azure Blob Storage"),
            CloudProviderKind::S3 => write!(f, "AWS S3"),
        }
    }
}

impl CloudProviderKind {
    pub fn check_cli_tools(&self) -> Result<(), Action> {
        match self {
            Self::Azure => {
                if Command::new("az").arg("--version").output().is_err() {
                    Err(Action::Error(
                        "Could not find requirement 'az'".to_string(),
                    ))
                } else {
                    Ok(())
                }
            },
            Self::Gcs => {
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
            Self::S3 => Err(Action::Error("S3 not implemented".to_string())),
        }
    }
}

impl From<String> for CloudProviderKind {
    fn from(cloud_provider_kind: String) -> Self {
        match cloud_provider_kind.as_str() {
            "Azure Blob Storage" => CloudProviderKind::Azure,
            "Google Cloud Storage" => CloudProviderKind::Gcs,
            "AWS S3" => CloudProviderKind::S3,
            _ => unimplemented!(),
        }
    }
}

impl From<&String> for CloudProviderKind {
    fn from(cloud_provider_kind: &String) -> Self {
        match cloud_provider_kind.as_str() {
            "Azure Blob Storage" => CloudProviderKind::Azure,
            "Google Cloud Storage" => CloudProviderKind::Gcs,
            "AWS S3" => CloudProviderKind::S3,
            _ => unimplemented!(),
        }
    }
}

