use std::sync::Arc;
use crossterm::event::KeyEvent;
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config as NucleoConfig, Nucleo};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Clear, List},
};
use std::result::Result;
use tui_textarea::TextArea;

use crate::{action::Action, app::Focus, config::Config, key::Key};

use super::filter_results::{ConnectionFilterResults, FilterResults, ViewerFilterResults};

pub trait Filter {
    fn default() -> Self where Self: Sized;
    fn engage_filter(&mut self, txt: Vec<String>, tree_items: Vec<String>) -> Result<Action, Action> {
        self.set_filter_result_items(tree_items);

        let number_of_columns = 1;

        let mut nucleo = Nucleo::new(
            NucleoConfig::DEFAULT,
            Arc::new(|| {}),
            None,
            number_of_columns,
        );

        // Send the strings to search through to the matcher
        let injector = nucleo.injector();

        let filtered_result_items = self.get_filter_result_items();

        for (id, string) in filtered_result_items
            .iter()
            .enumerate()
        {
            // Only the strings assigned to row in the closure below are matched on,
            // so it's possible to pass an identifier in.
            let item = (id, string.to_owned());

            injector.push(item, |(_id, string), row| {
                // The size of this array is determined by number_of_columns
                let str_clone = string.clone();
                row[0] = str_clone.into()
            });
        }

        if let Some(search_term) = txt.last() {
            nucleo.pattern.reparse(
                0,
                search_term,
                CaseMatching::Ignore,
                Normalization::Smart,
                false,
            );
        }

        // The search is initialised here...

        // ...but actually begins here
        let _status = nucleo.tick(500);

        // Snapshot contains the current set of results
        let snapshot = nucleo.snapshot();

        // Matching items are returned, ranked by highest score first.
        // These are just the items as pushed to the injector earlier.
        let matches: Vec<_> = snapshot.matched_items(..).collect();

        let mut data_list: Vec<String> = vec![];
        for item in matches {
            let (_, data) = item.data;

            data_list.push(data.to_string());
        }

        // set filtered items to the data list
        self.set_filter_result_filtered_items(data_list.clone());

        // gather the filtered items
        let filtered_items = self.get_filter_result_filtered_items().clone();

        // add filtered items to the results.items()
        let filter_result_items = self.get_filter_result_results().clone().items(filtered_items.clone());

        // set filtered results to the items above
        self.set_filter_result_results(filter_result_items);

        Ok(Action::Nothing)
        
    }
    fn switch_active_status(&mut self);
    fn get_filter_result_items(&mut self) -> &Vec<String>;
    fn set_filter_result_items(&mut self, tree_items: Vec<String>);
    fn set_filter_result_filtered_items(&mut self, data_list: Vec<String>);
    fn get_filter_result_filtered_items(&mut self) -> &Vec<String>;
    fn get_filter_result_results(&mut self) -> &List<'static>;
    fn set_filter_result_results(&mut self, filter_result_items: List<'static>);
    fn filter_results_handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action>;
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect, focus: Focus) -> Result<(), String>;
    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action>;
    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String>;
} 

pub struct ConnectionFilter {
    pub config: Config,
    pub active: bool,
    pub textarea: TextArea<'static>,
    pub filtered_results: Box<dyn FilterResults>,
}

pub struct ViewerFilter {
    pub config: Config,
    pub active: bool,
    pub textarea: TextArea<'static>,
    pub filtered_results: Box<dyn FilterResults>,
}


impl Filter for ConnectionFilter {
    fn default() -> Self where Self: Sized {
        Self { config: Config::default(), active: false, textarea: TextArea::default(), filtered_results: Box::new(ConnectionFilterResults::default()) }
    }

    fn switch_active_status(&mut self) {
        self.active = !self.active;
    }

    fn set_filter_result_items(&mut self, tree_items: Vec<String>) {
        self.filtered_results.set_items(tree_items);
    }

    fn get_filter_result_items(&mut self) -> &Vec<String> {
        self.filtered_results.get_items()
    }

    fn set_filter_result_filtered_items(&mut self, data_list: Vec<String>) {
        self.filtered_results.set_filtered_items(data_list);
    }

    fn get_filter_result_filtered_items(&mut self) -> &Vec<String> {
        self.filtered_results.get_filtered_items()
    }

    fn get_filter_result_results(&mut self) -> &List<'static> {
        self.filtered_results.get_results()
    }

    fn set_filter_result_results(&mut self, filter_result_items: List<'static>) {
        self.filtered_results.set_results(filter_result_items);
    }

    fn filter_results_handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action> {
        self.filtered_results.handle_key_event(key_event, focus)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<(), String> {
        let focused = matches!(focus, Focus::ConnectionsFilter);

        let [filter, list_res] =
            Layout::vertical([Constraint::Percentage(7), Constraint::Percentage(93)]).areas(area);

        self.textarea
            .set_cursor_line_style(ratatui::style::Style::default());
        self.textarea
            .set_placeholder_text("Add some text to begin filtering");
        self.textarea.set_style(Style::default().fg(Color::White));
        self.textarea
            .set_block(
                Block::bordered()
                    .title("Filter Connections")
                    .border_style(if focused {
                        Style::new().blue()
                    } else {
                        Style::default()
                    }),
            );
        if self.active {
            frame.render_widget(Clear, filter);
            frame.render_widget(&self.textarea, filter);
            frame.render_widget(Clear, list_res);
            self.filtered_results.draw(frame, list_res, focus)?;
        }
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        focus: Focus,
    ) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::ConnectionsFilter => {
                if key == self.config.key_config.exit {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.close_component {
                    self.active = !self.active;
                    Ok(Action::ChangeFocus(Focus::Connections))
                } else if matches!(key, Key::Char(_))
                    || [
                        self.config.key_config.backspace,
                        self.config.key_config.delete,
                        self.config.key_config.arrow_down,
                        self.config.key_config.arrow_up,
                        self.config.key_config.arrow_left,
                        self.config.key_config.arrow_right,
                    ]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    self.textarea.input(key_event);
                    Ok(Action::Filter(self.textarea.clone().into_lines()))
                } else if [
                    self.config.key_config.enter,
                    self.config.key_config.change_focus,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    Ok(Action::ChangeFocus(Focus::ConnectionFilterResults))
                } else {
                    Ok(Action::Nothing)
                }
            }
            _ => Ok(Action::Skip),
        }
    }

    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String> {
        self.config = config;
        self.filtered_results
            .register_config(self.config.clone(), focus)
    }
}

impl Filter for ViewerFilter {
    fn default() -> Self where Self: Sized {
        Self { config: Config::default(), active: false, textarea: TextArea::default(), filtered_results: Box::new(ViewerFilterResults::default()) }
    }

    fn set_filter_result_items(&mut self, tree_items: Vec<String>) {
        self.filtered_results.set_items(tree_items);
    }

    fn get_filter_result_items(&mut self) -> &Vec<String> {
        self.filtered_results.get_items()
    }

    fn set_filter_result_filtered_items(&mut self, data_list: Vec<String>) {
        self.filtered_results.set_filtered_items(data_list);
    }

    fn get_filter_result_filtered_items(&mut self) -> &Vec<String> {
        self.filtered_results.get_filtered_items()
    }

    fn get_filter_result_results(&mut self) -> &List<'static> {
        self.filtered_results.get_results()
    }

    fn set_filter_result_results(&mut self, filter_result_items: List<'static>) {
        self.filtered_results.set_results(filter_result_items);
    }

    fn filter_results_handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action> {
        self.filtered_results.handle_key_event(key_event, focus)
    }

    fn switch_active_status(&mut self) {
        self.active = !self.active;
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<(), String> {
        let focused = matches!(focus, Focus::ViewerFilter);
        // let [content, _] =
        //     Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        // let [connections, _viewer] =
        //     Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        let [filter, list_res] =
            Layout::vertical([Constraint::Percentage(7), Constraint::Percentage(93)]).areas(area);

        self.textarea
            .set_cursor_line_style(ratatui::style::Style::default());
        self.textarea
            .set_placeholder_text("Add some text to begin filtering");
        self.textarea.set_style(Style::default().fg(Color::White));
        self.textarea.set_block(
            Block::bordered()
                .title("Filter CloudFS Results")
                .border_style(if focused {
                    Style::new().blue()
                } else {
                    Style::default()
                }),
        );
        if self.active {
            frame.render_widget(Clear, filter);
            frame.render_widget(&self.textarea, filter);
            frame.render_widget(Clear, list_res);
            self.filtered_results.draw(frame, list_res, focus)?;
        }
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        focus: crate::app::Focus,
    ) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::ViewerFilter => {
                if key == self.config.key_config.exit {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.close_component {
                    self.active = !self.active;
                    Ok(Action::ChangeFocus(Focus::Viewer))
                } else if matches!(key, Key::Char(_))
                    || [
                        self.config.key_config.backspace,
                        self.config.key_config.delete,
                        self.config.key_config.arrow_down,
                        self.config.key_config.arrow_up,
                        self.config.key_config.arrow_left,
                        self.config.key_config.arrow_right,
                    ]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    self.textarea.input(key_event);
                    Ok(Action::Filter(self.textarea.clone().into_lines()))
                } else if [
                    self.config.key_config.enter,
                    self.config.key_config.change_focus,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    Ok(Action::ChangeFocus(Focus::ViewerFilterResults))
                } else {
                    Ok(Action::Nothing)
                }
            }
            _ => Ok(Action::Skip),
        }
    }

    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String> {
        self.config = config;
        self.filtered_results
            .register_config(self.config.clone(), focus)
    }
}
