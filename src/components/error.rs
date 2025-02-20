use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
};
use std::result::Result;
use tui_popup::Popup;

use crate::{action::Action, app::Focus, config::Config};

use super::Component;

#[derive(Debug, Default, Clone)]
pub struct ErrorComponent {
    pub config: Config,
    pub message: String,
}

impl Component for ErrorComponent {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<(), String> {
        let focused = matches!(focus, Focus::Error);
        let [content, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let error = Popup::new(self.message.as_ref()).style(Style::new().red().on_black());

        if focused {
            frame.render_widget(&error, content);
        }
        Ok(())
    }

    fn report_error(&mut self, message: String) -> Result<(), String> {
        self.message = [message, "Press any key to continue".to_string()].join(" -- ");
        Ok(())
    }

    fn register_config(
        &mut self,
        config: crate::config::Config,
        _focus: Focus,
    ) -> Result<(), String> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        _key_event: crossterm::event::KeyEvent,
        focus: Focus,
    ) -> Result<Action, Action> {
        match focus {
            Focus::Error => Ok(Action::ChangeFocus(Focus::Connections)),
            _ => Ok(Action::Skip),
        }
    }
}
