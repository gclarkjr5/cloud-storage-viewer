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
    initialize_logging().expect("error initializing logging");

    let mut app = App::new();
    app.run()?;

    Ok(())
}
