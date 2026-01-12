use cloud_provider_config::CloudProviderConfig;
use key_config::KeyConfig;


pub mod cloud_provider_config;
pub mod cloud_connection;
pub mod key_config;
pub mod cloud_provider_kind;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub key_config: KeyConfig,
    pub cloud_provider_config: CloudProviderConfig,
}

fn is_directory(loc: &str) -> bool {
    let x = true;
    println!("Is a directory {:?}", loc);

    x

    
}

pub trait CloudFS {
    // fn verify_cli_tooling(&self, selection: Vec<String>) -> Result<CloudProvider, Action>;
    // list available connections for a give cloud provider
    // GCP -> Projects
    // Azure -> Subscriptions
    // AWS -> TBD
    fn list_connections(self) -> Vec<String>;

    // list top-level cloud objects holding storage
    // GCP -> Buckets
    // Azure -> Storage Accounts (+ containers)
    // AWS -> Buckets
    fn list_storage(self, connection: str) -> Vec<String>;

    // list files/directories within the storage objects
    // GCP -> Directories + Files
    // Azure -> Storage Containers -> Directories + Files
    fn ls(&self, _connection: &str, _storage_object: &str, loc: &str) -> Vec<String> {
        if !is_directory(loc) {
            // not a directory, so nothing to list
            // return early
        }

        // actual listing
        vec!["test".to_string()]
    }

    // download the selected file from storage to local resource
    fn download(self, loc: str);
    
}
