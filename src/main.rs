use std::time::{Duration, Instant};

use app::CurrentScreen;
use crossterm::event::KeyEventKind;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::layout::Position;
use ratatui::{crossterm, Terminal};

mod app;
mod ui;

use crate::app::App;
use crate::ui::ui;

fn main() -> std::io::Result<()> {
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
            // if let Event::Key(key) = crossterm::event::read()? {

            let update = match crossterm::event::read()? {
                Event::Key(key) => match key.kind {
                    KeyEventKind::Release => {
                        continue;
                    }
                    _ => match app.current_screen {
                        CurrentScreen::Viewer => match key.code {
                            KeyCode::Char('q') | KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                return Ok(())
                            }
                            KeyCode::End | KeyCode::Char('j')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.viewer_state.select_last()
                            }
                            KeyCode::Home | KeyCode::Char('k')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.viewer_state.select_first()
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.viewer_state.key_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.viewer_state.key_up(),
                            KeyCode::Left | KeyCode::Char('h') => app.viewer_state.key_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.viewer_state.key_right(),
                            KeyCode::Char('\n' | ' ') => app.viewer_state.toggle_selected(),
                            KeyCode::Esc => app.viewer_state.select(Vec::new()),
                            KeyCode::PageDown => app.viewer_state.scroll_down(3),
                            KeyCode::PageUp => app.viewer_state.scroll_up(3),
                            KeyCode::Enter => {
                                // app.add_items(Some(app.state.viewer.selected().to_vec()));
                                app.list_items(Some(app.viewer_state.selected().to_vec()));
                                let selected = app.viewer_state.selected().to_vec();
                                app.viewer_state.open(selected)
                            }
                            KeyCode::Char('L') => {
                                app.increase_results_page();
                                app.list_items(Some(app.viewer_state.selected().to_vec()))
                            }
                            KeyCode::Tab => {
                                app.toggle_screen();
                                true
                            }
                            _ => false,
                        },
                        CurrentScreen::Connections => match key.code {
                            KeyCode::Char('q') | KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                return Ok(())
                            }
                            KeyCode::End | KeyCode::Char('j')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.connection_state.select_last()
                            }
                            KeyCode::Home | KeyCode::Char('k')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.connection_state.select_first()
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.connection_state.key_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.connection_state.key_up(),
                            KeyCode::Left | KeyCode::Char('h') => app.connection_state.key_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.connection_state.key_right(),
                            KeyCode::Char('\n' | ' ') => app.connection_state.toggle_selected(),
                            KeyCode::Esc => app.connection_state.select(Vec::new()),
                            KeyCode::PageDown => app.connection_state.scroll_down(3),
                            KeyCode::PageUp => app.connection_state.scroll_up(3),
                            KeyCode::Enter => {
                                // app.add_items(Some(app.connection_state.selected().to_vec()));
                                app.list_items(Some(app.connection_state.selected().to_vec()));
                                let selected = app.connection_state.selected().to_vec();
                                app.connection_state.open(selected)
                            }
                            KeyCode::Tab => {
                                app.toggle_screen();
                                true
                            }
                            KeyCode::Char('a') => app.activate_connection(Some(
                                app.connection_state.selected().to_vec(),
                            )),

                            _ => false,
                        },
                    },
                },
                Event::Mouse(mouse) => match app.current_screen {
                    CurrentScreen::Connections => match mouse.kind {
                        MouseEventKind::ScrollDown => app.connection_state.scroll_down(1),
                        MouseEventKind::ScrollUp => app.connection_state.scroll_up(1),
                        MouseEventKind::Down(_button) => app
                            .connection_state
                            .click_at(Position::new(mouse.column, mouse.row)),
                        _ => false,
                    },
                    CurrentScreen::Viewer => match mouse.kind {
                        MouseEventKind::ScrollDown => app.viewer_state.scroll_down(1),
                        MouseEventKind::ScrollUp => app.viewer_state.scroll_up(1),
                        MouseEventKind::Down(_button) => app
                            .viewer_state
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