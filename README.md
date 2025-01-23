# Cloud Storage Viewer

## The Why

I hate browsing object storage from the browser. I have to do it often. I want to do more things in Rust. So I created this RataTUI (Terminal User Interface) for it.

## The How

At the moment, this is **ONLY** available for Google Cloud Storage. So **ONLY** use that one. All others will exit with an **unimplemented** error.

It uses 2 Goolge CLI tools which you will need:

- **gcloud**: for configuration detection and modification
  - **for now, you must configure your gcloud configuration outside of this tool (this feature to come in the future)**
  - helpful commands:
    - `gcloud config configurations list`: list information regarding your current configurations
    - `gcloud config configurations (create/delete/rename)`: perform actions against existing configurations
- **gsutil**: for listing data out of GCS

### Downloading

I recommend downloading the binary (only built toward an MacOS Apple Silicon M3 at the moment. More to come...).

You can also build from source:

1. Git clone this repo
2. cd to the repo
3. cargo build --release
  - you will need **cargo** for this, which also requires **rust**


## Etc.

Feel free to contribute. There is a ton of room for improvement, like:

- other cloud implementations
- making it async
- fixing bugs
- refactoring

Additionally, please report any features/bugs using the Github Issues. I will try my best to get around to them, but understand I do this as a **HOBBY**.

