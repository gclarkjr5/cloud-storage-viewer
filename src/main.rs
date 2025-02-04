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
