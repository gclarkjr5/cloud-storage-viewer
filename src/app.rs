use std::io::Result;
use std::time::Duration;

use crossterm::event::{Event, KeyEvent, MouseEvent};

use super::components::connections::Connections;
use super::components::viewer::Viewer;
use crate::action::Action;
use crate::components::connection_filter::ConnectionFilter;
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
    ConnectionsFilter,
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
            component.init()?;
        }

        // time to work
        loop {
            // draw terminal
            self.render(&mut tui)?;

            // after drawing, handle terminal events
            match self.handle_events()? {
                Action::Quit => break,
                Action::ChangeFocus(focus) => self.change_focus(focus),
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
                    self.change_focus(Focus::Viewer)
                }
                Action::ActivateConfig(selection) => {
                    self.config.cloud_config.activate_config(selection)?;
                    for component in self.components.iter_mut() {
                        component.register_config(self.config.clone())?;
                    }
                }
                Action::Nothing => (),
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
        }
        Ok(res)
    }

    pub fn change_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }

    pub fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.terminal.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area(), self.focus) {
                    eprintln!("Failed to draw: {:?}", err);
                }
            }
        })?;
        Ok(())
    }
}
