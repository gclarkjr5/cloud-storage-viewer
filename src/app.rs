use std::result::Result;
use std::time::Duration;

use crossterm::event::{Event, KeyEvent, MouseEvent};
use tracing::info;

use super::components::connections::Connections;
use super::components::viewer::Viewer;
use crate::action::Action;
use crate::components::error::ErrorComponent;
use crate::components::footer::Footer;
use crate::components::{Component as Comp, TreeComponent};
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
                Box::new(Connections::default()),
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
        info!("Cloud Storage Viewer TUI started");
        let mut tui = Tui::new()?;
        tui.enter()?;
        tui.clear()?;

        for component in self.components.iter_mut() {
            component.register_config(self.config.clone(), self.focus)?;
            component.init()?;

            let component_name = &component.name();
            info!("Initialized and registerd default config for component {component_name:?}");
        }

        // time to work
        loop {
            // draw terminal
            self.render(&mut tui)?;

            // after drawing, handle terminal events
            match self.handle_events() {
                Ok(act) => match act {
                    Action::Quit => break,
                    Action::ChangeFocus(focus) => self.change_focus(focus),
                    // Action::ListCloudProvider(cloud_provider_config) => {
                    //     self.config.cloud_provider_config = cloud_provider_config;
                    //     for component in self.components.iter_mut() {
                    //         component.register_config(self.config.clone(), self.focus)?;
                    //     }
                    // }
                    Action::ListConfiguration(buckets) => {
                        // self.config.cloud_provider_config = cloud_provider_config.clone();
                        self.change_focus(Focus::Viewer);
                        let selection = format!("{}", self.config.cloud_provider_config);
                        for component in self.components.iter_mut() {
                            component.register_config(self.config.clone(), self.focus)?;
                            if let Some(tree_component) = component.as_any_mut().downcast_mut::<Viewer>() {
                                match tree_component.list_item(
                                    buckets.clone(),
                                    vec![selection.clone()],
                                    self.focus,
                                ) {
                                    Ok(_) => Ok(()),
                                    Err(act) => match act {
                                        Action::Error(e) => Err(e),
                                        _ => Ok(()),
                                    },
                                }?;
                            } else if let Some(tree_component) = component.as_any_mut().downcast_mut::<Footer>() {
                                match tree_component.list_item(
                                    buckets.clone(),
                                    vec![selection.clone()],
                                    self.focus,
                                ) {
                                    Ok(_) => Ok(()),
                                    Err(act) => match act {
                                        Action::Error(e) => Err(e),
                                        _ => Ok(()),
                                    },
                                }?;
                            }
                        }
                    }
                    Action::ActivateConfig(cloud_provider_config) => {
                        self.config.cloud_provider_config = cloud_provider_config;
                        // self.config.cloud_provider_config.activate_config(cloud_provider_config)?;
                        for component in self.components.iter_mut() {
                            component.register_config(self.config.clone(), self.focus)?;
                        }
                    }
                    Action::SelectFilteredItem(item, focus) => {
                        self.change_focus(focus);
                        for component in self.components.iter_mut() {
                            // component.select_item(&item, self.focus)?;
                            if let Some(tree_component) = component.as_any_mut().downcast_mut::<Connections>() {
                                tree_component.select_item(&item, self.focus)?;
                            } else if let Some(tree_component) = component.as_any_mut().downcast_mut::<Viewer>() {
                                tree_component.select_item(&item, self.focus)?;
                            }
                        }
                    }
                    _ => (),
                },
                Err(act) => match act {
                    Action::Error(message) => {
                        for component in self.components.iter_mut() {
                            component.report_error(message.clone())?;
                        }
                        self.change_focus(Focus::Error);
                    }
                    _ => break,
                },
            };
        }

        tui.exit()?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<Action, Action> {
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
                    Err(Action::Error(message))
                }
            },
            Err(_) => {
                let message = "Error polling for events".to_string();
                Err(Action::Error(message))
            }
        }
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<Action, Action> {
        // convert key event into Key
        let mut act = Action::Nothing;

        // handle event for components
        for component in self.components.iter_mut() {
            let res = component.handle_key_event(key_event, self.focus)?;

            if !matches!(res, Action::Skip) {
                act = res
            }
        }

        Ok(act)
    }

    fn handle_mouse_events(&mut self, mouse_event: MouseEvent) -> Result<Action, Action> {
        let res = Action::Nothing;
        // handle event for components
        for component in self.components.iter_mut() {
            component.handle_mouse_event(mouse_event, self.focus)?;
        }
        Ok(res)
    }

    pub fn change_focus(&mut self, focus: Focus) {
        let initial_focus = self.focus;
        info!("Changing focus from {initial_focus:?} to {focus:?}");
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
