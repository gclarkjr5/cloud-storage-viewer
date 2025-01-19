use std::io::Result;
use std::time::Duration;

use crossterm::event::{Event, KeyEvent, MouseEvent};

use super::components::connections::Connections;
use super::components::viewer::Viewer;
use crate::action::Action;
use crate::components::footer::Footer;
use crate::components::Component as Comp;
use crate::config::cloud_config::CloudProvider;
use crate::config::Config;
use crate::key::Key;
use crate::tui::Tui;

#[derive(Debug, Clone, Copy)]
pub enum Focus {
    Connections,
    Viewer,
}

#[must_use]
pub struct App {
    pub _should_quit: bool,
    pub components: Vec<Box<dyn Comp>>,
    pub focus: Focus,
    pub config: Config,
    // pub cloud: Cloud,
    // pub connections: Connections,
    // pub viewer: Viewer,
    // pub footer: Footer
}

impl App {
    pub fn new() -> Self {
        Self {
            _should_quit: false,
            components: vec![
                Box::new(Connections::new()),
                Box::new(Viewer::default()),
                Box::new(Footer::default()),
            ],
            focus: Focus::Connections,
            config: Config::default(),
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        // start the TUI
        let mut tui = Tui::new()?;
        tui.enter()?;
        tui.terminal.clear()?;

        // regisration
        // for component in self.components.iter_mut() {
        //     component.register_action_handler(self.action_tx.clone())?;
        // }
        self.config.init()?;
        for component in self.components.iter_mut() {
            component.register_config(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init()?;
        }

        // time to work
        loop {
            // draw terminal
            self.render(&mut tui)?;
            // tui.terminal.draw(|frame| {
            //     // draw holding application onto the frames of the terminal
            //     if let Err(err) = self.draw(frame) {
            //         println!("Erorr: {:?}", err);
            //         std::process::exit(1);
            //     };
            // })?;

            // after drawing, handle terminal events
            match self.handle_events()? {
                Action::Quit => break,
                Action::ChangeFocus => self.change_focus(),
                Action::ListCloudProvider(cloud_config) => {
                    self.config.cloud_config = cloud_config;
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone())?;
                    }
                }
                Action::ListConfiguration(cloud_config, selection, buckets) => {
                    self.config.cloud_config = cloud_config;
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone())?;
                        component.list_items(buckets.clone(), selection.clone())?;
                    }
                    self.change_focus()
                }
                Action::ActivateConfig(selection) => {
                    self.config.cloud_config.activate_config(selection)?;
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone())?;
                    }
                }
                Action::Nothing => (),
                // _ => unimplemented!(),
            };
            // if self
            //     .handle_events()
            //     .is_ok_and(|action| matches!(action, Action::Quit))
            // {
            //     break;
            // };

            // self.handle_events();
            // self.handle_actions();
            // if self.should_quit {
            //     tui.exit();
            //     break;
            // }
        }

        tui.exit()?;
        Ok(())
    }

    fn _list_item(&mut self, selection: Vec<String>) {
        // self.activate_connection(selection);

        // if someone just hit enter on Connections do nothing
        let final_node = selection.last().unwrap().as_str();
        if matches!(final_node, "Connections") {
            return;
        }

        // get second node, if its a cloud, set active cloud and list items
        let second_node = selection.get(1).unwrap().clone().into();
        match second_node {
            CloudProvider::Azure(config) => self
                .config
                .cloud_config
                .set_active_cloud(CloudProvider::Azure(config)),
            CloudProvider::Gcs(config) => self
                .config
                .cloud_config
                .set_active_cloud(CloudProvider::Gcs(config)),
            CloudProvider::S3(config) => self
                .config
                .cloud_config
                .set_active_cloud(CloudProvider::S3(config)),
        }
    }

    fn handle_events(&mut self) -> Result<Action> {
        if crossterm::event::poll(Duration::from_millis(250))? {
            match crossterm::event::read()? {
                Event::Key(key) => self.handle_key_events(key),
                Event::Mouse(mouse) => self.handle_mouse_events(mouse),
                Event::Resize(_, _) => Ok(Action::Nothing),
                _ => Ok(Action::Quit),
            }
        } else {
            Ok(Action::Nothing)
        }
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<Action> {
        // convert key event into Key
        let key: Key = key_event.into();

        // let mut action: Action = Action::Nothing;
        let mut res = Action::Nothing;
        // handle event for components
        for component in self.components.iter_mut() {
            let act = component.handle_key_event(key, self.focus)?;
            if act
                .clone()
                .is_some_and(|a| matches!(a, Action::ActivateConfig(_selection)))
            {
                res = act.unwrap();
            } else if act.is_some() {
                res = act.unwrap()
            }
        }
        Ok(res)
    }

    fn handle_mouse_events(&mut self, mouse_event: MouseEvent) -> Result<Action> {
        let res = Action::Nothing;
        // handle event for components
        for component in self.components.iter_mut() {
            component.handle_mouse_event(mouse_event, self.focus)?;
            // let act = component.handle_key_event(key, self.focus)?;
            // if act
            //     .clone()
            //     .is_some_and(|a| matches!(a, Action::ActivateConfig(_selection)))
            // {
            //     res = act.unwrap();
            // } else if act.is_some() {
            //     res = act.unwrap()
            // }
        }
        Ok(res)
        // }
    }

    pub fn change_focus(&mut self) {
        match self.focus {
            Focus::Connections => self.focus = Focus::Viewer,
            Focus::Viewer => self.focus = Focus::Connections,
        }
    }

    pub fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.terminal.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area(), self.focus) {
                    eprintln!("Failed to draw: {:?}", err);
                    // let _ = self
                    //     .action_tx
                    //     .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }

    //     // let action_tx = self.action_tx.clone();
    //     loop {
    //         // self.handle_events(&mut tui).await?;
    //         // self.handle_actions(&mut tui)?;
    //         // if self.should_suspend {
    //         //     tui.suspend()?;
    //         //     action_tx.send(Action::Resume)?;
    //         //     action_tx.send(Action::ClearScreen)?;
    //         //     // tui.mouse(true);
    //         //     tui.enter()?;
    //         // } else if self.should_quit {
    //         //     tui.stop()?;
    //         //     break;
    //         // }
    //         // let timeout =
    //         //     debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
    //         if crossterm::event::poll(timeout)? {
    //             let update = match crossterm::event::read()? {
    //                 Event::Key(key) => match key.kind {
    //                     KeyEventKind::Release => {
    //                         continue;
    //                     }
    //                     _ => match app.focus {
    //                         Focus::Viewer => match key.code {
    //                             KeyCode::Char('q') | KeyCode::Char('c')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 return Ok(())
    //                             }
    //                             KeyCode::End | KeyCode::Char('j')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 app.viewer.state.select_last()
    //                             }
    //                             KeyCode::Home | KeyCode::Char('k')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 app.viewer.state.select_first()
    //                             }
    //                             KeyCode::Char('l')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 // if the page index does increase
    //                                 if app.viewer.increase_results_page().is_some() {
    //                                     app.list_items(
    //                                         Some(vec![app.viewer.results_pager.paged_item.clone()]),
    //                                         "previous_page",
    //                                     );
    //                                     true
    //                                 } else {
    //                                     false
    //                                 }
    //                             }
    //                             KeyCode::Char('h')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 if app.viewer.decrease_results_page().is_some() {
    //                                     app.list_items(
    //                                         Some(vec![app.viewer.results_pager.paged_item.clone()]),
    //                                         "previous_page",
    //                                     );
    //                                     true
    //                                 } else {
    //                                     false
    //                                 }
    //                             }
    //                             KeyCode::Down | KeyCode::Char('j') => app.viewer.state.key_down(),
    //                             KeyCode::Up | KeyCode::Char('k') => app.viewer.state.key_up(),
    //                             KeyCode::Left | KeyCode::Char('h') => app.viewer.state.key_left(),
    //                             KeyCode::Right | KeyCode::Char('l') => app.viewer.state.key_right(),
    //                             KeyCode::Char('\n' | ' ') => app.viewer.state.toggle_selected(),
    //                             KeyCode::Esc => app.viewer.state.select(Vec::new()),
    //                             KeyCode::PageDown => app.viewer.state.scroll_down(3),
    //                             KeyCode::PageUp => app.viewer.state.scroll_up(3),
    //                             KeyCode::Enter => {
    //                                 // app.add_items(Some(app.state.viewer.selected().to_vec()));
    //                                 app.list_items(
    //                                     Some(app.viewer.state.selected().to_vec()),
    //                                     "request",
    //                                 );
    //                                 let selected = app.viewer.state.selected().to_vec();
    //                                 app.viewer.state.open(selected)
    //                             }
    //                             KeyCode::Tab => {
    //                                 app.toggle_screen();
    //                                 true
    //                             }
    //                             _ => false,
    //                         },
    //                         Focus::Connections => match key.code {
    //                             KeyCode::Char('q') | KeyCode::Char('c')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 return Ok(())
    //                             }
    //                             KeyCode::End | KeyCode::Char('j')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 app.connections.state.select_last()
    //                             }
    //                             KeyCode::Home | KeyCode::Char('k')
    //                                 if key.modifiers.contains(KeyModifiers::CONTROL) =>
    //                             {
    //                                 app.connections.state.select_first()
    //                             }
    //                             KeyCode::Down | KeyCode::Char('j') => {
    //                                 app.connections.state.key_down()
    //                             }
    //                             KeyCode::Up | KeyCode::Char('k') => app.connections.state.key_up(),
    //                             KeyCode::Left | KeyCode::Char('h') => {
    //                                 app.connections.state.key_left()
    //                             }
    //                             KeyCode::Right | KeyCode::Char('l') => {
    //                                 app.connections.state.key_right()
    //                             }
    //                             KeyCode::Char('\n' | ' ') => {
    //                                 app.connections.state.toggle_selected()
    //                             }
    //                             KeyCode::Esc => app.connections.state.select(Vec::new()),
    //                             KeyCode::PageDown => app.connections.state.scroll_down(3),
    //                             KeyCode::PageUp => app.connections.state.scroll_up(3),
    //                             KeyCode::Enter => {
    //                                 app.list_items(
    //                                     Some(app.connections.state.selected().to_vec()),
    //                                     "request",
    //                                 );
    //                                 let selected = app.connections.state.selected().to_vec();
    //                                 app.connections.state.open(selected)
    //                             }
    //                             KeyCode::Tab => {
    //                                 app.toggle_screen();
    //                                 true
    //                             }
    //                             KeyCode::Char('a') => {
    //                                 app.connections.activate_connection(Some(
    //                                     app.connections.state.selected().to_vec(),
    //                                 ));
    //                                 app.viewer = Viewer::new(
    //                                     app.connections.active.clone().unwrap().as_str(),
    //                                 );
    //                                 true
    //                             }

    //                             _ => false,
    //                         },
    //                     },
    //                 },
    //                 Event::Mouse(mouse) => match app.focus {
    //                     Focus::Connections => match mouse.kind {
    //                         MouseEventKind::ScrollDown => app.connections.state.scroll_down(1),
    //                         MouseEventKind::ScrollUp => app.connections.state.scroll_up(1),
    //                         MouseEventKind::Down(_button) => app
    //                             .connections
    //                             .state
    //                             .click_at(Position::new(mouse.column, mouse.row)),
    //                         _ => false,
    //                     },
    //                     Focus::Viewer => match mouse.kind {
    //                         MouseEventKind::ScrollDown => app.viewer.state.scroll_down(1),
    //                         MouseEventKind::ScrollUp => app.viewer.state.scroll_up(1),
    //                         MouseEventKind::Down(_button) => app
    //                             .viewer
    //                             .state
    //                             .click_at(Position::new(mouse.column, mouse.row)),
    //                         _ => false,
    //                     },
    //                 },
    //                 Event::Resize(_, _) => true,
    //                 _ => false,
    //             };
    //             if update {
    //                 debounce.get_or_insert_with(Instant::now);
    //             }
    //         }
    //         // if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
    //         //     let before = Instant::now();
    //         //     terminal.draw(|frame| {
    //         //         ui(frame, &mut app, &before);
    //         //     })?;

    //         //     debounce = None;
    //         // }
    //     }
    //     // tui.exit()?;
    //     // Ok(())
    // }

    // pub fn empty_viewer(&mut self) {
    //     self.viewer = Viewer::new();
    // }

    // pub fn list_items(&mut self, path_identifier: Option<Vec<String>>, action: &str) -> bool {
    //     match path_identifier {
    //         None => {}

    //         Some(path) => match self.focus {
    //             Focus::Connections => {
    //                 // list items from connections
    //                 let items = self.connections.list_items(path.clone());

    //                 match items {
    //                     None => false, // current account did not match the requested one
    //                     Some(_) => {
    //                         // we hit Enter on a connection that lists buckets
    //                         self.viewer =
    //                             Viewer::new(self.connections.active.clone().unwrap().as_str());
    //                         let viewer_root = self.connections.active.clone();
    //                         // init pager
    //                         let selection = path
    //                             .iter()
    //                             .last()
    //                             .expect("error getting selected item")
    //                             .as_str();

    //                         self.viewer.results_pager.paged_item = selection.to_string();

    //                         self.viewer.list_items(
    //                             vec![viewer_root.clone().unwrap().to_string()],
    //                             "request",
    //                         );
    //                         self.focus = Focus::Viewer;
    //                         self.viewer
    //                             .state
    //                             .open(vec![viewer_root.unwrap().to_string()]);
    //                         true
    //                     }
    //                 };
    //             }
    //             Focus::Viewer => {
    //                 self.viewer.list_items(path, action);
    //             }
    //         },
    //     }
    //     true
    // }
}
