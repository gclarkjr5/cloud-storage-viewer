use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Clear},
};
use tui_textarea::TextArea;

use crate::{action::Action, app::Focus, config::Config, key::Key};

use super::{viewer_filter_results::ViewerFilterResults, Component};

#[derive(Debug, Default)]
pub struct ViewerFilter {
    pub config: Config,
    pub active: bool,
    pub textarea: TextArea<'static>,
    pub filtered_results: ViewerFilterResults,
}

impl Component for ViewerFilter {
    fn init(&mut self) -> std::io::Result<()> {
        self.filtered_results.init()?;
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> std::io::Result<()> {
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
    ) -> std::io::Result<Option<crate::action::Action>> {
        let key: Key = key_event.into();
        match focus {
            Focus::ViewerFilter => {
                if key == self.config.key_config.exit {
                    Ok(Some(Action::Quit))
                } else if key == self.config.key_config.close_component {
                    self.active = !self.active;
                    Ok(Some(Action::ChangeFocus(Focus::Viewer)))
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
                    Ok(Some(Action::Filter(self.textarea.clone().into_lines())))
                } else if [
                    self.config.key_config.enter,
                    self.config.key_config.change_focus,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    Ok(Some(Action::ChangeFocus(Focus::ViewerFilterResults)))
                } else {
                    Ok(Some(Action::Nothing))
                }
            }
            _ => Ok(None),
        }
    }

    fn register_config(&mut self, config: Config, focus: Focus) -> std::io::Result<()> {
        self.config = config;
        self.filtered_results
            .register_config(self.config.clone(), focus)?;
        Ok(())
    }
}
