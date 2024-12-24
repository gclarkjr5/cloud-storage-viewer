use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use std::time::Instant;
use tui_tree_widget::Tree;

use crate::app::CurrentScreen;
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
    let connections_widget = Tree::new(&app.connection_items)
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
        &mut app.connection_state,
    );

    if let CurrentScreen::Connections = app.current_screen {
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
            &mut app.connection_state,
        );
    }

    ////////////////////////////////////////

    // viewer
    let viewer_widget = Tree::new(&app.viewer_items)
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

    frame.render_stateful_widget(viewer_widget.clone(), viewer, &mut app.viewer_state);

    if let CurrentScreen::Viewer = app.current_screen {
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

        frame.render_stateful_widget(highlight_viewer, viewer, &mut app.viewer_state);
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

    // let footer_block = Block::default()
    //     .borders(Borders::ALL)
    //     .style(Style::default());

    let active_connection = footer_chunks[0];
    let commands = footer_chunks[1];
    let quit_and_close = footer_chunks[2];

    let active_connection_widget = Paragraph::new(Line::from(vec![app
        .active_connection
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

    let commands_widget = match app.current_screen {
        CurrentScreen::Connections => {
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
        CurrentScreen::Viewer => {
            let viewer_commands = vec![
                "Viewer Commands: ".into(),
                "Switch to Connections=".into(),
                "[Tab] ".blue(),
                "List Items=".into(),
                "[Enter]".blue(),
            ];
            Paragraph::new(Line::from(viewer_commands)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Viewer Commands")
                    .style(Style::default()),
            )
        }
    };

    frame.render_widget(commands_widget, commands);

    let quit_and_close_widget = Paragraph::new(Line::from(vec!["'q' / ".red(), "Ctrl + C".red()]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Quit/Close")
                .style(Style::default()),
        );

    frame.render_widget(quit_and_close_widget, quit_and_close);

    /////////////////////////////////////

    // let mode_footer = Paragraph::new(Line::from(current_navigation_text))
    // .block(Block::default().borders(Borders::ALL));

    // let current_keys_hint = {
    //     match app.current_screen {
    //         CurrentScreen::Main => Span::styled(
    //             "(q) to quit / (e) to make new pair",
    //             Style::default().fg(Color::Red),
    //         ),
    //         CurrentScreen::Editing => Span::styled(
    //             "(ESC) to cancel/(Tab) to switch boxes/enter to complete",
    //             Style::default().fg(Color::Red),
    //         ),
    //         CurrentScreen::Exiting => Span::styled(
    //             "(q) to quit / (e) to make new pair",
    //             Style::default().fg(Color::Red),
    //         ),
    //     }
    // };

    // frame.render_widget(footer_widget, footer);

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
}
