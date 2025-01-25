use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, List, ListDirection, ListState},
};

use crate::{action::Action, app::Focus, config::Config, key::Key};

use super::Component;

#[derive(Debug, Default, Clone)]
pub struct ConnectionFilterResults {
    pub config: Config,
    pub items: Vec<String>,
    pub filtered_items: Vec<String>,
    pub results: List<'static>,
    pub state: ListState,
}

impl Component for ConnectionFilterResults {
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> std::io::Result<()> {
        let focused = matches!(focus, Focus::ConnectionFilterResults);
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
    ) -> std::io::Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key_event: crossterm::event::KeyEvent,
        focus: Focus,
    ) -> std::io::Result<Option<crate::action::Action>> {
        let key: Key = key_event.into();
        match focus {
            Focus::ConnectionFilterResults => {
                if key == self.config.key_config.exit {
                    Ok(Some(Action::Quit))
                } else if key == self.config.key_config.enter {
                    let item_idx = self.state.selected().unwrap();
                    let item = self.filtered_items[item_idx].clone();
                    Ok(Some(Action::SelectFilteredItem(item, Focus::Viewer)))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.select_previous();
                    Ok(Some(Action::Nothing))
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.select_next();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.change_focus {
                    Ok(Some(Action::ChangeFocus(Focus::ConnectionsFilter)))
                } else {
                    Ok(Some(Action::Nothing))
                }
            }
            _ => Ok(None),
        }
    }
}
