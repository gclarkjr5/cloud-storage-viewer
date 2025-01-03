// use color_eyre::owo_colors::OwoColorize;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use std::time::Instant;
use tui_tree_widget::Tree;

use crate::app::Focus;
use crate::App;

pub fn ui(frame: &mut Frame, app: &mut App, before: &Instant) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    let content = main_chunks[0];
    let footer = main_chunks[1];

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(15), Constraint::Min(1)])
        .split(content);

    let connections = content_chunks[0];
    let viewer = content_chunks[1];

    // connections
    let connections_widget = Tree::new(&app.connections.items)
        .expect("all item identifieers are unique")
        .block(Block::bordered().title("Cloud Connections"))
        .experimental_scrollbar(Some(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
        ));

    frame.render_stateful_widget(
        connections_widget.clone(),
        connections,
        &mut app.connections.state,
    );

    if let Focus::Connections = app.focus {
        let highlight_connections = connections_widget
            .block(
                Block::bordered()
                    .title("Cloud Connections")
                    .border_style(Style::new().blue()),
            )
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(
            highlight_connections,
            connections,
            &mut app.connections.state,
        );
    }

    ////////////////////////////////////////

    // viewer
    let viewer_widget = Tree::new(&app.viewer.items)
        .expect("all item identifiers are unique")
        .block(
            Block::bordered().title("Cloud Storage Viewer"), // .title_bottom(format!("{:?}", app.state)),
        )
        .experimental_scrollbar(Some(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .track_symbol(None)
                .end_symbol(None),
        ));

    frame.render_stateful_widget(viewer_widget.clone(), viewer, &mut app.viewer.state);

    if let Focus::Viewer = app.focus {
        let highlight_viewer = viewer_widget
            .block(
                Block::bordered()
                    .title("Cloud Storage Viewer")
                    .border_style(Style::new().blue()),
            )
            .highlight_style(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(highlight_viewer, viewer, &mut app.viewer.state);
    }
    ///////////////////////////////////////

    // footer
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Min(1),
            Constraint::Percentage(15),
        ])
        .split(footer);

    let active_connection = footer_chunks[0];
    let commands = footer_chunks[1];
    let quit_and_close = footer_chunks[2];

    let active_connection_widget = Paragraph::new(Line::from(vec![app
        .connections
        .active
        .clone()
        .unwrap()
        .green()]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Active Connections")
            .style(Style::default()),
    );

    frame.render_widget(active_connection_widget, active_connection);

    let commands_widget = match app.focus {
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
            let mut viewer_commands = vec![
                "Switch to Connections=".into(),
                "[Tab] ".blue(),
                "List Items=".into(),
                "[Enter] ".blue(),
            ];
            if app.viewer.results_pager.num_pages > 1 {
                viewer_commands.push("Next Page=".into());
                viewer_commands.push("[Ctrl+l] ".blue());
                viewer_commands.push("Previous Page=".into());
                viewer_commands.push("[Ctrl+h] ".blue());
            }
            Paragraph::new(Line::from(viewer_commands)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Viewer Commands")
                    .style(Style::default()),
            )
        }
    };

    frame.render_widget(commands_widget, commands);

    let quit_and_close_widget = Paragraph::new(Line::from(vec!["Ctrl + C".red(), " / 'q'".red()]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Quit/Close")
                .style(Style::default()),
        );

    frame.render_widget(quit_and_close_widget, quit_and_close);

    /////////////////////////////////////

    let last_render_took = before.elapsed();
    // Performance info in top right corner
    {
        let text = format!(
            " {} {last_render_took:?} {:.1} FPS",
            frame.count(),
            1.0 / last_render_took.as_secs_f64()
        );
        #[allow(clippy::cast_possible_truncation)]
        let area = Rect {
            y: 0,
            height: 1,
            x: frame.area().width.saturating_sub(text.len() as u16),
            width: text.len() as u16,
        };
        frame.render_widget(
            Span::styled(text, Style::new().fg(Color::Black).bg(Color::Gray)),
            area,
        );
    }
    if app.viewer.results_pager.num_pages > 1 {
        let paging_commands = format!(
            "currently paging: {}
            page: {} of {}
            showing: {} of {}",
            app.viewer.results_pager.paged_item,
            app.viewer.results_pager.page_idx + 1,
            app.viewer.results_pager.num_pages,
            app.viewer.results_pager.results_per_page,
            app.viewer.results_pager.total_results,
        );

        #[allow(clippy::cast_possible_truncation)]
        let paging_area = Rect {
            y: content.height - 2,
            height: 10,
            x: frame
                .area()
                .width
                .saturating_sub(paging_commands.len() as u16),
            width: paging_commands.len() as u16,
        };
        frame.render_widget(
            Span::styled(
                paging_commands,
                Style::new().fg(Color::Black).bg(Color::Gray),
            ),
            // Paragraph::new(text),
            paging_area,
        );
    }
}
