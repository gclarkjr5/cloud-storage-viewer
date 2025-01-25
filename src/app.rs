use std::result::Result;
use std::time::Duration;

use crossterm::event::{Event, KeyEvent, MouseEvent};

use super::components::connections::Connections;
use super::components::viewer::Viewer;
use crate::action::Action;
use crate::components::error::ErrorComponent;
use crate::components::footer::Footer;
use crate::components::Component as Comp;
use crate::config::Config;
use crate::tui::Tui;

#[derive(Debug, Clone, Copy)]
pub enum Focus {
    Connections,
    Viewer,
    ConnectionsFilter,
    ViewerFilter,
    ConnectionFilterResults,
    ViewerFilterResults,
    Error,
}

#[must_use]
pub struct App {
    pub _should_quit: bool,
    pub components: Vec<Box<dyn Comp>>,
    pub focus: Focus,
    pub config: Config,
}

impl App {
    pub fn new() -> Self {
        Self {
            _should_quit: false,
            components: vec![
                Box::new(Connections::new()),
                Box::new(Viewer::default()),
                Box::new(Footer::default()),
                Box::new(ErrorComponent::default()),
            ],
            focus: Focus::Connections,
            config: Config::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        // start the TUI
        let mut tui = Tui::new()?;
        tui.enter()?;
        tui.clear()?;

        // regisration
        // for component in self.components.iter_mut() {
        //     component.register_action_handler(self.action_tx.clone())?;
        // }
        self.config.init()?;
        for component in self.components.iter_mut() {
            component.register_config(self.config.clone(), self.focus)?;
            component.init()?;
        }

        // time to work
        loop {
            // draw terminal
            self.render(&mut tui)?;

            // after drawing, handle terminal events
            match self.handle_events()? {
                Action::Quit => break,
                Action::Error(message) => {
                    for component in self.components.iter_mut() {
                        component.report_error(message.clone())?;
                    }
                    self.change_focus(Focus::Error);
                }
                Action::ChangeFocus(focus) => self.change_focus(focus),
                Action::ListCloudProvider(cloud_config) => {
                    self.config.cloud_config = cloud_config;
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone(), self.focus)?;
                    }
                }
                Action::ListConfiguration(cloud_config, selection, buckets) => {
                    self.config.cloud_config = cloud_config;
                    self.change_focus(Focus::Viewer);
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone(), self.focus)?;
                        component.list_items(buckets.clone(), selection.clone(), self.focus)?;
                    }
                }
                Action::ActivateConfig(selection) => {
                    self.config.cloud_config.activate_config(selection)?;
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone(), self.focus)?;
                    }
                }
                Action::SelectFilteredItem(item, focus) => {
                    self.change_focus(focus);
                    for component in self.components.iter_mut() {
                        component.select_item(&item, self.focus)?;
                    }
                }
                Action::Nothing => (),
                Action::Filter(_) => (),
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

    fn handle_events(&mut self) -> Result<Action, String> {
        match crossterm::event::poll(Duration::from_millis(250)) {
            Ok(_) => match crossterm::event::read() {
                Ok(event) => match event {
                    Event::Key(key) => self.handle_key_events(key),
                    Event::Mouse(mouse) => self.handle_mouse_events(mouse),
                    Event::Resize(_, _) => Ok(Action::Nothing),
                    _ => Ok(Action::Quit),
                },
                Err(_) => {
                    let message = "Error reading events".to_string();
                    Err(message)
                }
            },
            Err(_) => {
                let message = "Error polling for events".to_string();
                Err(message)
            }
        }
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<Action, String> {
        // convert key event into Key
        // let key: Key = key_event.into();

        let mut res = Action::Nothing;
        // handle event for components
        for component in self.components.iter_mut() {
            let act = component.handle_key_event(key_event, self.focus)?;
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

    fn handle_mouse_events(&mut self, mouse_event: MouseEvent) -> Result<Action, String> {
        let res = Action::Nothing;
        // handle event for components
        for component in self.components.iter_mut() {
            component.handle_mouse_event(mouse_event, self.focus)?;
        }
        Ok(res)
    }

    pub fn change_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }

    pub fn render(&mut self, tui: &mut Tui) -> Result<(), String> {
        match tui.terminal.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area(), self.focus) {
                    eprintln!("Failed to draw: {:?}", err);
                }
            }
        }) {
            Ok(_) => Ok(()),
            Err(_) => {
                let message = "Error drawing to terminal".to_string();
                Err(message)
            }
        }
    }
}
