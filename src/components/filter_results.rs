use crossterm::event::KeyEvent;
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, List, ListDirection, ListState},
};
use std::result::Result;

use crate::{action::Action, app::Focus, config::Config, key::Key};

pub trait FilterResults: std::fmt::Debug {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect, focus: Focus) -> Result<(), String>;
    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action>;
    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String>;
    fn get_items(&mut self) -> &Vec<String>;
    fn set_items(&mut self, tree_items: Vec<String>);
    fn set_filtered_items(&mut self, data_list: Vec<String>);
    fn get_filtered_items(&mut self) -> &Vec<String>;
    fn set_results(&mut self, filter_result_items: List<'static>);
    fn get_results(&mut self) -> &List<'static>;
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionFilterResults {
    pub config: Config,
    pub items: Vec<String>,
    pub filtered_items: Vec<String>,
    pub results: List<'static>,
    pub state: ListState,
}

#[derive(Debug, Default, Clone)]
pub struct ViewerFilterResults {
    pub config: Config,
    pub items: Vec<String>,
    pub filtered_items: Vec<String>,
    pub results: List<'static>,
    pub state: ListState,
}

impl FilterResults for ConnectionFilterResults {
    fn set_items(&mut self, tree_items: Vec<String>) {
        self.items = tree_items;
    }

    fn get_items(&mut self) -> &Vec<String> {
        &self.items
    }

    fn set_filtered_items(&mut self, data_list: Vec<String>) {
        self.filtered_items = data_list;
    }

    fn get_filtered_items(&mut self) -> &Vec<String> {
        &self.filtered_items
    }

    fn get_results(&mut self) -> &List<'static> {
        &self.results
    }

    fn set_results(&mut self, filter_result_items: List<'static>) {
        self.results = filter_result_items;
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<(), String> {
        let focused = matches!(focus, Focus::ConnectionFilterResults);
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
    ) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::ConnectionFilterResults => {
                if key == self.config.key_config.exit {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.enter {
                    let item_idx = self.state.selected().unwrap();
                    let item = self.filtered_items[item_idx].clone();
                    Ok(Action::SelectFilteredItem(item, Focus::Connections))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.select_previous();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.select_next();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.change_focus {
                    Ok(Action::ChangeFocus(Focus::ConnectionsFilter))
                } else {
                    Ok(Action::Nothing)
                }
            }
            _ => Ok(Action::Skip),
        }
    }
    
}

impl FilterResults for ViewerFilterResults {
    fn set_items(&mut self, tree_items: Vec<String>) {
        self.items = tree_items;
    }

    fn get_items(&mut self) -> &Vec<String> {
        &self.items
    }

    fn set_filtered_items(&mut self, data_list: Vec<String>) {
        self.filtered_items = data_list;
    }

    fn get_filtered_items(&mut self) -> &Vec<String> {
        &self.filtered_items
    }

    fn get_results(&mut self) -> &List<'static> {
        &self.results
    }

    fn set_results(&mut self, filter_result_items: List<'static>) {
        self.results = filter_result_items;
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<(), String> {
        let focused = matches!(focus, Focus::ViewerFilterResults);
        let list = self
            .results
            .clone()
            .block(
                Block::bordered()
                    .title("CloudFS Results Filtered")
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
    ) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::ViewerFilterResults => {
                if key == self.config.key_config.exit {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.enter {
                    let item_idx = self.state.selected().unwrap();
                    let item = self.filtered_items[item_idx].clone();
                    Ok(Action::SelectFilteredItem(item, Focus::Viewer))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.select_previous();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.select_next();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.change_focus {
                    Ok(Action::ChangeFocus(Focus::ViewerFilter))
                } else {
                    Ok(Action::Nothing)
                }
            }
            _ => Ok(Action::Skip),
        }
    }
}

