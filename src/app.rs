use std::fmt;
use std::result::Result;
use std::time::Duration;

use crossterm::event::{Event, KeyEvent, MouseEvent};
use ego_tree::NodeId;
use tracing::info;

use super::components::connections::Connections;
use super::components::viewer::Viewer;
use crate::action::Action;
use crate::components::connections::ConnectionComponentSelection;
use crate::components::error::ErrorComponent;
use crate::components::footer::Footer;
use crate::components::{Component as Comp, TreeComponent};
use crate::config::Config;
use crate::config::cloud_provider_config::cloud_provider_connection::CloudConnection;
use crate::config::cloud_provider_config::cloud_provider_kind::CloudProviderKind;
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
    // pub error_component: ErrorComponent,
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
            // error_component: ErrorComponent::default(),
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
            component.register_config(&self.config, self.focus)?;
            component.init(&self.config)?;

            let component_name = &component.name();
            info!("Initialized and registerd default config for component {component_name:?}");
        }

        // self.error_component.register_config(&self.config, self.focus)?;
        // self.error_component.init(&self.config)?;

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
                    Action::ConnectionList(connection_selection) => {
                        self.config.app_selection = connection_selection.clone();
                        match self.connection_ls(connection_selection) {
                            Err(e) => if let Action::Error(e) = e {
                                self.change_focus(Focus::Error);
                                for component in self.components.iter_mut() {
                                    component.report_error(&e)?;
                                }
                            }
                            Ok(_) => {
                                for component in self.components.iter_mut() {
                                    component.register_config(&self.config, self.focus)?;
                                }
                                
                            }
                        }
                        // let gcp = self.config.cloud_provider_config.gcs.clone();
                        // info!("{gcp:?}");
                        // self.change_focus(Focus::Viewer);
                        // let selection = format!("{}", self.config.cloud_provider_config);
                        // // let active = self.config.cloud_provider_config.active_cloud_connection.clone();
                        // // info!("Attempting to select the following in Viewer {active:?}");
                        // for component in self.components.iter_mut() {
                        //     component.register_config(&self.config, self.focus)?;
                        // }
                        //     if let Some(tree_component) = component.as_any_mut().downcast_mut::<Viewer>() {
                        //         match tree_component.list_item(
                        //             buckets.clone(),
                        //             vec![selection.clone()],
                        //             self.focus,
                        //         ) {
                        //             Ok(_) => Ok(()),
                        //             Err(act) => match act {
                        //                 Action::Error(e) => Err(e),
                        //                 _ => Ok(()),
                        //             },
                        //         }?;
                        //     } else if let Some(tree_component) = component.as_any_mut().downcast_mut::<Footer>() {
                        //         match tree_component.list_item(
                        //             buckets.clone(),
                        //             vec![selection.clone()],
                        //             self.focus,
                        //         ) {
                        //             Ok(_) => Ok(()),
                        //             Err(act) => match act {
                        //                 Action::Error(e) => Err(e),
                        //                 _ => Ok(()),
                        //             },
                        //         }?;
                        //     }
                        // }
                    }
                    Action::Activate(connection_selection) => {
                        // self.config.cloud_provider_config = cloud_provider_config;
                        // self.config.cloud_provider_config.activate_config(cloud_provider_config)?;
                        match self.activate(connection_selection) {
                            Err(e) => if let Action::Error(e) = e {
                                self.change_focus(Focus::Error);
                                for component in self.components.iter_mut() {
                                    component.report_error(&e)?;
                                }
                            },
                            Ok(_) => {
                                for component in self.components.iter_mut() {
                                    component.register_config(&self.config, self.focus)?;
                                }
                                
                            }
                            
                        }


                            // for component in self.components.iter_mut() {
                            //     component.register_config(&self.config, self.focus)?;
                            // }
                        // match self.activate(connection_selection) {
                        //     Err(_e) => (),
                        //     Ok(_) => ()
                            
                        // }
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
                        self.change_focus(Focus::Error);
                        for component in self.components.iter_mut() {
                            component.report_error(&message)?;
                        }
                        // self.error_component.report_error(message.clone())?;
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
        self.focus = focus;

    }

    pub fn render(&mut self, tui: &mut Tui) -> Result<(), String> {
        match tui.terminal.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area(), self.focus, &self.config) {
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

    pub fn connection_ls(
        &mut self,
        connection_selection: Vec<String>,
    ) -> Result<Action, Action> {

        self.config.cloud_provider_config.ls(connection_selection)
        // match connection_selection.cloud_provider_connection {
        //     None => {
        //         // if request_path.is_empty() {
        //         //     return Err(Action::Error("Cannot List Top Level".to_string()))
        //         // }
        //         let cp = connection_selection.cloud_provider_kind;

        //         info!("Listing Cloud Provider {cp:?}");
        //         let listing_output = self.config.cloud_provider_config.ls(&connection_selection)?;
        //         Ok(Action::Nothing)
        //         // info!("Searching for Tree Node to append to {request_path:?}");

                // let found_node = self.find_node_to_append(selection)?;
                // match found_node {
                //     None => {
                //         info!("No Tree Node Identified");
                //         Ok(Action::Nothing)
                //     },
                //     Some(nid) => {
                //         info!("Tree Node Identified. Creating Stateful Tree for {request_path:?}");

                //         self.create_nodes(nid, &listing_request.provider_kind)?;
                //         info!("Connection Tree Nodes created");

                //         Ok(Action::Nothing)
                //         // self.items =
                //         //     util::make_tree_items(self.tree.nodes(), &mut self.results_pager, Focus::Connections);
                //         // info!("Recursive Tree Nodes created");
                //         // self.state.open(selection.to_vec());
                //         // Ok(listing_output)
                //     }
                // }
        //     },
        //     Some(_) => {
        //         let listing_output = self.config.cloud_provider_config.ls(&connection_selection)?;
        //         Ok(listing_output)
        //     }
        // }
        // // find the node to append to
        // info!("Searching for Tree Node to append to {request_path:?}");
        // let found_node = self.find_node_to_append(selection)?;

        // // if empty dont do anything
        // match found_node {
        //     None => {
        //         info!("No Tree Node Identified");
        //         Ok(Action::Nothing)
        //     },
        //     Some(nid) => {
        //         info!("Tree Node Identified");

        //         // let cloud_provider = self.config.cloud_provider_config.get_cloud_provider(cloud_provider).expect("Error returning cloud provider from conns");

        //         let listing_output = self.config.cloud_provider_config.ls(&listing_request)?;

                
                

        //         info!("Creating Stateful Tree for {request_path:?}");

        //         self.create_nodes(nid, &listing_request.provider_kind)?;
        //         // self.config.cloud_provider_config.create_nodes(&mut self.tree, nid, &cloud_provider)
        //         // cloud_provider.create_nodes(&mut self.tree, nid)?;


        //         info!("Connection Tree Nodes created");

        //         self.items =
        //             util::make_tree_items(self.tree.nodes(), &mut self.results_pager, Focus::Connections);
        //         info!("Recursive Tree Nodes created");
        //         self.state.open(selection.to_vec());
        //         Ok(listing_output)
        //         // Ok(Action::Nothing)
        //     }
        // }
    }

    // pub fn activate(&mut self, connection_selection: ConnectionComponentSelection) -> Result<(), Action> {
    pub fn activate(&mut self, selection: Vec<String>) -> Result<(), Action> {

        self.config.cloud_provider_config.activate(selection)
        //     Ok(_) => Ok(()),
        //     Err(e) => match e {
        //         Action::Error(e) => {
                    
        //         }
        //         _ => Ok(())
        //     }
            
        // }
        // if self.config.cloud_provider_config.active_cloud_connection.is_none() {
        //     return Err(Action::Error("Nothing active connection, please first list a Cloud Provider".to_string()))
        // }

        // info!("{connection_selection}");
        // match connection_selection.cloud_provider_connection {
        //     None => {
        //         // No account means we just find the currently active account within
        //         // the cloud provider and make that the active connection
        //         match connection_selection.cloud_provider_kind {
        //             CloudProviderKind::S3 => Err(Action::Error("Activation Not Implemented".to_string())),
        //             CloudProviderKind::Azure => {
        //                 let conf = self.config.cloud_provider_config.azure.iter().find(|conf| conf.is_active);
        //                 self.config.cloud_provider_config.active_cloud_connection = conf.map(|c| CloudConnection::Azure(c.clone()));
        //                 info!("Active Cloud Connection: {:?}", self.config.cloud_provider_config.active_cloud_connection);
        //                 Ok(())
        //             },
        //             CloudProviderKind::Gcs => {
        //                 let conf = self.config.cloud_provider_config.gcs.iter().find(|conf| conf.is_active);
        //                 self.config.cloud_provider_config.active_cloud_connection = conf.map(|c| CloudConnection::Gcs(c.clone()));
        //                 info!("Active Cloud Connection: {:?}", self.config.cloud_provider_config.active_cloud_connection);
        //                 Ok(())
        //             },
        //         }
                
        //     }
        //     Some(acc) => {
        //         // An account means that it was selected by the user to change to
        //         match connection_selection.cloud_provider_kind {
        //             CloudProviderKind::S3 => Err(Action::Error("".to_string())),
        //             CloudProviderKind::Azure => {
        //                 // mutate all to change to requested account
        //                 self.config.cloud_provider_config.azure.iter_mut().for_each(|conf| {
        //                     conf.is_active = conf.name == acc;
        //                 });

        //                 let conf = self.config.cloud_provider_config.azure.iter().find(|conf| conf.is_active);
        //                 self.config.cloud_provider_config.active_cloud_connection = conf.map(|c| CloudConnection::Azure(c.clone()));

        //                 info!("Active Cloud Connection: {:?}", self.config.cloud_provider_config.active_cloud_connection);
        //                 Ok(())
        //             },
        //             CloudProviderKind::Gcs => {
        //                 // mutate all to change to requested account
        //                 self.config.cloud_provider_config.gcs.iter_mut().for_each(|conf| {
        //                     conf.is_active = conf.name == acc;
        //                 });

        //                 // return newly active and set as active
        //                 let conf = self.config.cloud_provider_config.gcs.iter().find(|conf| conf.is_active);
        //                 self.config.cloud_provider_config.active_cloud_connection = conf.map(|c| CloudConnection::Gcs(c.clone()));

        //                 info!("Active Cloud Connection: {:?}", self.config.cloud_provider_config.active_cloud_connection);
        //                 Ok(())
        //             },
        //         }
                
        //     }
        // }
    }

    // fn create_nodes(&mut self, node_id: NodeId, cloud_provider: &CloudProviderKind) -> Result<(), Action> {
    //         match cloud_provider {
    //             CloudProviderKind::Azure => {
    //                 self.config.cloud_provider_config.azure.iter().for_each(|config| {
    //                     let res = config.name.to_string();

    //                     match self.tree.get_mut(node_id) {
    //                         None => (),
    //                         Some(mut tree) => {tree.append(res);}
    //                     }
    //                 });
    //                 Ok(())
                
    //             },
    //             CloudProviderKind::Gcs => {
    //                 self.config.cloud_provider_config.gcs.iter().for_each(|config| {
    //                     let res = config.name.to_string();

    //                     match self.tree.get_mut(node_id) {
    //                         None => (),
    //                         Some(mut tree) => {tree.append(res);}
    //                     }
    //                 });
    //                 Ok(())
    //             }
    //             CloudProviderKind::S3 => Err(Action::Error("S3 not implemented".to_string())),
    //         }
        
    // }
}


