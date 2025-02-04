use std::io::{stdout, Stdout};
use std::result::Result;

use crossterm::{
    event::{EnableMouseCapture, KeyEvent, MouseEvent},
    terminal::EnterAlternateScreen,
};
use ratatui::backend::CrosstermBackend as Backend;

#[derive(Clone, Debug)]
pub enum _Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self, String> {
        let terminal = ratatui::Terminal::new(Backend::new(stdout()));

        match terminal {
            Ok(term) => Ok(Self { terminal: term }),
            Err(_) => {
                let message = "Error attaching a new Ratatui terminal".to_string();
                Err(message)
            }
        }
    }

    pub fn enter(&mut self) -> Result<(), String> {
        match crossterm::terminal::enable_raw_mode() {
            Ok(_) => {
                match crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture) {
                    Ok(e) => Ok(e),
                    Err(_) => Err("error executing the terminal".to_string()),
                }
            }
            Err(_) => Err("error enabling terminal raw mode".to_string()),
        }
    }

    pub fn exit(&mut self) -> Result<(), String> {
        match crossterm::terminal::disable_raw_mode() {
            Ok(()) => {
                match crossterm::execute!(
                    self.terminal.backend_mut(),
                    crossterm::terminal::LeaveAlternateScreen,
                    crossterm::event::DisableMouseCapture
                ) {
                    Ok(()) => match self.terminal.show_cursor() {
                        Ok(()) => Ok(()),
                        Err(_) => {
                            let message = format!("Error clearing terminal");
                            Err(message)
                        }
                    },
                    Err(_) => {
                        let message = format!("Error mutating Crossterm backend on exit");
                        Err(message)
                    }
                }
            }
            Err(_) => {
                let message = format!("Error disabling raw mode");
                return Err(message);
            }
        }
    }

    pub fn clear(&mut self) -> Result<(), String> {
        match self.terminal.clear() {
            Ok(t) => Ok(t),
            Err(_) => {
                let message = "Error clearing terminal".to_string();
                Err(message)
            }
        }
    }
}
