use std::io::Result;

use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::Focus,
    config::{
        cloud_config::{CloudProvider, GcsConfig},
        Config,
    },
};

use super::Component;

#[derive(Debug)]
pub struct Footer {
    // pub _key_config: KeyConfig,
    pub config: Config,
    // pub active: Option<String>,
}

impl Default for Footer {
    fn default() -> Self {
        Self::new()
    }
}

impl Footer {
    pub fn new() -> Self {
        Self {
            // _key_config: KeyConfig::default(),
            config: Config::default(),
        }
    }
}

impl Component for Footer {
    fn init(&mut self) -> Result<()> {
        // self.config.get_cloud_config();
        // self.active = Some("active connection".to_string());
        Ok(())
    }

    fn register_config(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        focus: crate::app::Focus,
    ) -> Result<()> {
        let [_, footer] = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let [active_connection, commands, quit_and_close] = Layout::horizontal([
            Constraint::Percentage(15),
            Constraint::Min(1),
            Constraint::Percentage(15),
        ])
        .areas(footer);

        let active_config = format!("{}", self.config.cloud_config);

        let active_connection_widget = Paragraph::new(Line::from(vec![active_config.green()]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Active Connections")
                    .style(Style::default()),
            );

        frame.render_widget(active_connection_widget, active_connection);

        let commands_widget = match focus {
            Focus::Connections => {
                let connection_commands = vec![
                    "Switch to Viewer=".into(),
                    "[Tab] ".blue(),
                    "List Items=".into(),
                    "[Enter] ".blue(),
                    "Activate Account=".into(),
                    "[a]".blue(),
                ];
                Paragraph::new(Line::from(connection_commands)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Connection Commands")
                        .style(Style::default()),
                )
            }
            Focus::Viewer => {
                let viewer_commands = vec![
                    "Switch to Connections=".into(),
                    "[Tab] ".blue(),
                    "List Items=".into(),
                    "[Enter] ".blue(),
                ];
                // if app.viewer.results_pager.num_pages > 1 {
                //     viewer_commands.push("Next Page=".into());
                //     viewer_commands.push("[Ctrl+l] ".blue());
                //     viewer_commands.push("Previous Page=".into());
                //     viewer_commands.push("[Ctrl+h] ".blue());
                // }
                Paragraph::new(Line::from(viewer_commands)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Viewer Commands")
                        .style(Style::default()),
                )
            }
        };

        frame.render_widget(commands_widget, commands);

        let quit_and_close_widget =
            Paragraph::new(Line::from(vec!["Ctrl + C".red(), " / 'q'".red()])).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Quit/Close")
                    .style(Style::default()),
            );

        frame.render_widget(quit_and_close_widget, quit_and_close);
        Ok(())
    }
}
