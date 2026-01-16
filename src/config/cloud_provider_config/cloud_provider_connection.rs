use std::fmt;

use tracing::info;

use crate::{action::Action, config::cloud_provider_config::cloud_provider_kind::CloudProviderKind, util};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3Config {
    pub name: String,
    pub data: Option<Vec<u8>>
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AzureConfig {
    pub name: String,
    pub is_active: bool,
    pub data: Option<Vec<u8>>
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GcsConfig {
    pub name: String,
    pub is_active: bool,
    pub data: Option<Vec<u8>>
}

impl GcsConfig {
    pub fn ls(&self) -> Result<Vec<u8>, Action> {
        let output = util::cli_command("gsutil", &vec!["ls"])?; 
        //     Err(_e) => Err(Action::Error("cli ls issue".to_string())),
        //     Ok(output) => {
        //         self.data = Some(output.clone());
        Ok(output)
        //     }
        // }

        // self.data = Some(output);

        // Ok(())
        // Ok(output)

        // Ok(Action::ListConnection(
        //     self.config.cloud_provider_config.,
        //     output,
        // ))
        
    }
}


#[derive(Debug, Clone)]
pub enum CloudConnection {
    S3(S3Config),
    Azure(AzureConfig),
    Gcs(GcsConfig),
}

impl fmt::Display for CloudConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloudConnection::Azure(conf) => write!(f, "{}({})", conf.name, CloudProviderKind::Azure),
            CloudConnection::Gcs(conf) => write!(f, "{}({})", conf.name, CloudProviderKind::Gcs),
            CloudConnection::S3(conf) => write!(f, "{}({})", conf.name, CloudProviderKind::S3),
        }
    }
    // fn fmt(cloud_connection: CloudConnection) -> Self {
    // }
}

impl From<CloudConnection> for String {
    fn from(cloud_connection: CloudConnection) -> Self {
        match cloud_connection {
            CloudConnection::Azure(_) => "Azure Blob Storage".to_string(),
            CloudConnection::Gcs(_) => "Google Cloud Storage".to_string(),
            CloudConnection::S3(_) => "AWS S3".to_string(),
        }
    }
}

impl CloudConnection {
    pub fn name(&self) -> String {
        match self {
            CloudConnection::S3(conf) => conf.name.clone(),
            CloudConnection::Azure(conf) => conf.name.clone(),
            CloudConnection::Gcs(conf) => conf.name.clone(),
        }
    }
    pub fn set_data(&self, data: Vec<u8>) -> Option<CloudConnection> {
        match self {
            CloudConnection::S3(conf) => {
                Some(CloudConnection::S3(S3Config { name: conf.name.clone(), data: Some(data) }))
            },
            CloudConnection::Azure(conf) => {
                Some(CloudConnection::Azure(AzureConfig { name: conf.name.clone(), is_active: conf.is_active, data: Some(data) }))
            },
            CloudConnection::Gcs(conf) => {
                Some(CloudConnection::Gcs(GcsConfig { name: conf.name.clone(), is_active: conf.is_active, data: Some(data) }))
            },
        }
        
    }
}
