use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, List, ListDirection, ListState},
};
use std::result::Result;

use crate::{action::Action, app::Focus, config::Config, key::Key};

use super::Component;

#[derive(Debug, Default, Clone)]
pub struct ErrorComponent {
    pub config: Config,
    pub message: String,
}

impl Component for ErrorComponent {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<(), String> {
        let focused = matches!(focus, Focus::Error);
        // let [content, _] =
        //     Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        // let [connections, _viewer] =
        //     Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        // let [_, results] =
        //     Layout::vertical([Constraint::Percentage(7), Constraint::Percentage(93)]).areas(area);
        let list = self
            .results
            .clone()
            .block(
                Block::bordered()
                    .title("Connection Results Filtered")
                    .border_style(if focused {
                        Style::new().blue()
                    } else {
                        Style::default()
                    }),
            )
            .style(Style::new().bg(Color::Black))
            .highlight_style(if focused {
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, area, &mut self.state);
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
        key_event: crossterm::event::KeyEvent,
        focus: Focus,
    ) -> Result<Option<crate::action::Action>, String> {
        let key: Key = key_event.into();
        match focus {
            Focus::ConnectionFilterResults => {
                if key == self.config.key_config.exit {
                    Ok(Some(Action::Quit))
                } else {
                    Ok(Some(Action::Nothing))
                }
            }
            _ => Ok(None),
        }
    }
}
