use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Clear},
};
use tui_textarea::TextArea;

use super::Component;
use crate::{action::Action, app::Focus, config::Config};

#[derive(Debug, Default)]
pub struct ConnectionFilter {
    pub config: Config,
    pub active: bool,
}

impl Component for ConnectionFilter {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> std::io::Result<()> {
        let focused = matches!(focus, Focus::ConnectionsFilter);
        let [content, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let [connections, _] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        let [filter, _] =
            Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)])
                .areas(connections);
        // } else {
        //     [Rect::default(), connections]
        // };

        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(ratatui::style::Style::default());
        textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
        textarea.set_style(Style::default().fg(Color::LightRed));
        textarea.set_block(
            Block::bordered()
                .title("Filter Connex")
                .border_style(if focused {
                    Style::new().blue()
                } else {
                    Style::default()
                }),
        );
        // frame.render_widget(Clear, filter);
        if self.active {
            frame.render_widget(&textarea, filter);
        }
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key: crate::key::Key,
        focus: crate::app::Focus,
    ) -> std::io::Result<Option<crate::action::Action>> {
        match focus {
            Focus::ConnectionsFilter => {
                if [self.config.key_config.quit, self.config.key_config.exit]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    Ok(Some(Action::Quit))
                } else if key == self.config.key_config.filter {
                    self.active = !self.active;
                    Ok(Some(Action::ChangeFocus(Focus::Connections)))
                } else {
                    Ok(Some(Action::Nothing))
                }
            }
            _ => Ok(None),
        }
    }

    fn register_config(&mut self, config: Config) -> std::io::Result<()> {
        self.config = config;
        Ok(())
    }
}
