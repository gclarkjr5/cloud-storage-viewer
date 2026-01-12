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

#[derive(Debug, Clone)]
pub enum CloudConnection {
    S3(S3Config),
    Azure(AzureConfig),
    Gcs(GcsConfig),
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
}
