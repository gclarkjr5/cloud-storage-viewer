use std::time::{Duration, Instant};

use app::Focus;
use components::viewer::Viewer;
use crossterm::event::KeyEventKind;
use logging::initialize_logging;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::layout::Position;
use ratatui::{crossterm, Terminal};

mod app;
mod cli;
mod components;
mod config;
mod logging;
mod ui;

use crate::app::App;
use crate::ui::ui;

fn main() -> std::io::Result<()> {
    initialize_logging().expect("error initializing logging");
    // Terminal initialization
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // App
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

    let before = Instant::now();
    terminal.draw(|frame| ui(frame, &mut app, &before))?;

    let mut debounce: Option<Instant> = None;

    loop {
        let timeout = debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
        if crossterm::event::poll(timeout)? {
            let update = match crossterm::event::read()? {
                Event::Key(key) => match key.kind {
                    KeyEventKind::Release => {
                        continue;
                    }
                    _ => match app.focus {
                        Focus::Viewer => match key.code {
                            KeyCode::Char('q') | KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                return Ok(())
                            }
                            KeyCode::End | KeyCode::Char('j')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.viewer.state.select_last()
                            }
                            KeyCode::Home | KeyCode::Char('k')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.viewer.state.select_first()
                            }
                            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // if the page index does increase
                                if app.viewer.increase_results_page().is_some() {
                                    app.list_items(
                                        Some(vec![app.viewer.results_pager.paged_item.clone()]),
                                        "previous_page",
                                    );
                                    true
                                } else {
                                    false
                                }
                            }
                            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if app.viewer.decrease_results_page().is_some() {
                                    app.list_items(
                                        Some(vec![app.viewer.results_pager.paged_item.clone()]),
                                        "previous_page",
                                    );
                                    true
                                } else {
                                    false
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.viewer.state.key_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.viewer.state.key_up(),
                            KeyCode::Left | KeyCode::Char('h') => app.viewer.state.key_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.viewer.state.key_right(),
                            KeyCode::Char('\n' | ' ') => app.viewer.state.toggle_selected(),
                            KeyCode::Esc => app.viewer.state.select(Vec::new()),
                            KeyCode::PageDown => app.viewer.state.scroll_down(3),
                            KeyCode::PageUp => app.viewer.state.scroll_up(3),
                            KeyCode::Enter => {
                                // app.add_items(Some(app.state.viewer.selected().to_vec()));
                                app.list_items(
                                    Some(app.viewer.state.selected().to_vec()),
                                    "request",
                                );
                                let selected = app.viewer.state.selected().to_vec();
                                app.viewer.state.open(selected)
                            }
                            KeyCode::Tab => {
                                app.toggle_screen();
                                true
                            }
                            _ => false,
                        },
                        Focus::Connections => match key.code {
                            KeyCode::Char('q') | KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                return Ok(())
                            }
                            KeyCode::End | KeyCode::Char('j')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.connections.state.select_last()
                            }
                            KeyCode::Home | KeyCode::Char('k')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.connections.state.select_first()
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.connections.state.key_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.connections.state.key_up(),
                            KeyCode::Left | KeyCode::Char('h') => app.connections.state.key_left(),
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.connections.state.key_right()
                            }
                            KeyCode::Char('\n' | ' ') => app.connections.state.toggle_selected(),
                            KeyCode::Esc => app.connections.state.select(Vec::new()),
                            KeyCode::PageDown => app.connections.state.scroll_down(3),
                            KeyCode::PageUp => app.connections.state.scroll_up(3),
                            KeyCode::Enter => {
                                app.list_items(
                                    Some(app.connections.state.selected().to_vec()),
                                    "request",
                                );
                                let selected = app.connections.state.selected().to_vec();
                                app.connections.state.open(selected)
                            }
                            KeyCode::Tab => {
                                app.toggle_screen();
                                true
                            }
                            KeyCode::Char('a') => {
                                app.connections.activate_connection(Some(
                                    app.connections.state.selected().to_vec(),
                                ));
                                app.viewer =
                                    Viewer::new(app.connections.active.clone().unwrap().as_str());
                                true
                            }

                            _ => false,
                        },
                    },
                },
                Event::Mouse(mouse) => match app.focus {
                    Focus::Connections => match mouse.kind {
                        MouseEventKind::ScrollDown => app.connections.state.scroll_down(1),
                        MouseEventKind::ScrollUp => app.connections.state.scroll_up(1),
                        MouseEventKind::Down(_button) => app
                            .connections
                            .state
                            .click_at(Position::new(mouse.column, mouse.row)),
                        _ => false,
                    },
                    Focus::Viewer => match mouse.kind {
                        MouseEventKind::ScrollDown => app.viewer.state.scroll_down(1),
                        MouseEventKind::ScrollUp => app.viewer.state.scroll_up(1),
                        MouseEventKind::Down(_button) => app
                            .viewer
                            .state
                            .click_at(Position::new(mouse.column, mouse.row)),
                        _ => false,
                    },
                },
                Event::Resize(_, _) => true,
                _ => false,
            };
            if update {
                debounce.get_or_insert_with(Instant::now);
            }
        }
        if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
            let before = Instant::now();
            terminal.draw(|frame| {
                ui(frame, &mut app, &before);
            })?;

            debounce = None;
        }
    }
}
