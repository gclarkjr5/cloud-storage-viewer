use std::io::{stdout, Result, Stdout};

use crossterm::{
    event::{EnableMouseCapture, KeyEvent, MouseEvent},
    terminal::EnterAlternateScreen,
};
use ratatui::backend::CrosstermBackend as Backend;

#[derive(Clone, Debug)]
pub enum Event {
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
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: ratatui::Terminal::new(Backend::new(stdout()))?,
        })
    }

    pub fn enter(&mut self) -> std::io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture,);

        Ok(())
    }

    pub fn exit(&mut self) -> std::io::Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}
