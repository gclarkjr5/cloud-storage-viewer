#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

use logging::initialize_logging;
use std::result::Result;

mod action;
mod app;
mod components;
mod config;
mod key;
mod logging;
mod tui;
mod util;

use crate::app::App;

fn main() -> Result<(), String> {
    match initialize_logging() {
        Ok(_) => {
            let mut app = App::new();
            app.run()?;
            Ok(())
        }
        Err(_) => Err("error initializing logging".to_string()),
    }
}

// GCS structure
// // Project/Account -- gcloud config configurations list, activate, create
// // // Buckets -- gsutil ls
// // // // Filesystem -- gsutil ls {bucket} ...

// Azure structure
// // Azure Subscription -- az account list, set --subscription, `cannot create`
// // // Storage Accounts -- az storage account list --subscription {subscription} --query '[].name' --output yaml
// // // Resource Groups -- az group list
// // // // Storage Account -- az storage account list
// // // // // Containers -- [az storage account keys list -n {account_name}, az storage container list --account-name {name} --account-key {key}]
// // // // // // Filesystem (blobs) -- az storage blob list --container-name {container} --account-name {account} --account-key {key}

// S3 structure
// // Project/Account
// // // Buckets
// // // // Filesystem
