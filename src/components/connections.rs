use super::filter::{ConnectionFilter, Filter};
use super::results_pager::ResultsPager;
use super::{Component, TreeComponent};
use crossterm::event::{KeyEvent, MouseEventKind};
use ego_tree::{NodeId, Tree as ETree};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::block::Block;
use ratatui::widgets::{Clear, Scrollbar, ScrollbarOrientation};

use ratatui::Frame;
use tracing::{info};
use std::result::Result;
use std::vec;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::action::Action;
use crate::app::Focus;
use crate::config::Config;
use crate::config::cloud_provider_kind::CloudProviderKind;
use crate::key::Key;
use crate::util;

#[derive(Debug)]
pub struct Connections {
    pub state: TreeState<String>,
    pub tree: ETree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub config: Config,
    pub results_pager: ResultsPager,
    pub filter: Box<dyn Filter>,
}

impl Default for Connections {
    fn default() -> Self {
        Self {
            state: TreeState::default(),
            tree: ETree::new(String::new()),
            items: Vec::new(),
            config: Config::default(),
            results_pager: ResultsPager::default(),
            filter: Box::new(ConnectionFilter::default()),
        }
    }
}


impl Connections {
    pub fn activate(&mut self, selection: Vec<String>) -> Result<Action, Action> {
        // Remove Connections
        // first is Cloud Provider, then account (GCS/account_1, Azure/account_0, etc.)
        let request_path = &selection[1..];
        info!("User Request: Activate Connection {request_path:?}.");

        if request_path.is_empty() {
            // User tried to activate "Connections", do Nothing
            Ok(Action::Nothing)
        } else {
            // if the user tries to activate the cloud provider, then we will activate the cloud
            // provider and the currently active account for it
            let cloud_provider: CloudProviderKind = request_path[0].clone().into();
            if request_path.len() == 1 {
                
                self.config.cloud_provider_config.activate(&cloud_provider, None)?;
                Ok(Action::ActivateConfig(self.config.cloud_provider_config.clone()))
            } else {
                let account = request_path[1].to_string();
                self.config.cloud_provider_config.activate(&cloud_provider, Some(account))?;
                Ok(Action::ActivateConfig(self.config.cloud_provider_config.clone()))
                
            }
            // info!("Activating, but not yet");
            // Err(Action::Error("Not implemented".to_string()))
        }
    }

    // pub fn ls(
    pub fn list_cloud_provider(
        &mut self,
        selection: Vec<String>,
        focus: Focus,
    ) -> Result<Action, Action> {
        let request_path = &selection[1..];
        info!("User Request: List Connections for {request_path:?}");

        let cloud_provider_selection: String = selection[1].clone();
        let cloud_provider: CloudProviderKind = cloud_provider_selection.into();

        // verify that the selected connection is for an implemented cloud
        // technically should never happen, mainly to see how far development
        // of more cloud implementations has gotten
        self
            .config
            .cloud_provider_config
            .verify_implemented_cloud_provider(cloud_provider.clone())?;

        // find the node to append to
        info!("Searching for Tree Node to append to {request_path:?}");
        let found_node = self.find_node_to_append(&selection)?;

        // if empty dont do anything
        match found_node {
            None => {
                info!("No Tree Node Identified");
                Ok(Action::Nothing)
            },
            Some(nid) => {
                info!("Tree Node Identified");

                // let cloud_provider = self.config.cloud_provider_config.get_cloud_provider(cloud_provider).expect("Error returning cloud provider from conns");

                self.config.cloud_provider_config.list_connections(&cloud_provider)?;
                info!("Creating Stateful Tree for {request_path:?}");

                self.create_nodes(nid, &cloud_provider)?;
                // self.config.cloud_provider_config.create_nodes(&mut self.tree, nid, &cloud_provider)
                // cloud_provider.create_nodes(&mut self.tree, nid)?;


                info!("Connection Tree Nodes created");

                self.items =
                    util::make_tree_items(self.tree.nodes(), &mut self.results_pager, focus);
                info!("Recursive Tree Nodes created");
                self.state.open(selection.clone());
                Ok(Action::Nothing)
            }
        }
    }

    fn create_nodes(&mut self, node_id: NodeId, cloud_provider: &CloudProviderKind) -> Result<(), Action> {
            match cloud_provider {
                CloudProviderKind::Azure => {
                    self.config.cloud_provider_config.azure.iter().for_each(|config| {
                        let res = config.name.to_string();

                        match self.tree.get_mut(node_id) {
                            None => (),
                            Some(mut tree) => {tree.append(res);}
                        }
                    });
                    Ok(())
                
                },
                CloudProviderKind::Gcs => {
                    self.config.cloud_provider_config.gcs.iter().for_each(|config| {
                        let res = config.name.to_string();

                        match self.tree.get_mut(node_id) {
                            None => (),
                            Some(mut tree) => {tree.append(res);}
                        }
                    });
                    Ok(())
                }
                CloudProviderKind::S3 => Err(Action::Error("S3 not implemented".to_string())),
            }
        
    }

    pub fn list_configuration(&mut self, _path_identifier: Vec<String>) -> Result<Action, Action> {
        // set active cloud
        // self.config.cloud_provider_config.activate_config(path_identifier)?;

        // list items
        let output = util::cli_command("gsutil", &vec!["ls"])?;

        Ok(Action::ListConfiguration(
            self.config.cloud_provider_config.clone(),
            output,
        ))
    }
}

impl Component for Connections {
    fn name(&self) -> &str {
        "Connections"
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self) -> Result<(), String> {
        let mut tree = ETree::new("Cloud Providers".to_string());

        self.config
            .cloud_provider_config
            .all_cloud_providers()
            .iter()
            .for_each(|cloud| {
                tree.root_mut().append(cloud.to_string());
            });

        let mut items = vec![];

        tree.nodes()
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                if let Ok(mut ti) = TreeItem::new(val.clone(), val.clone(), vec![]) {
                    let mut results_pager = self.results_pager.clone();

                    util::add_children(node, &mut ti, &mut results_pager, Focus::Connections);
                    items.push(ti);
                }
            });

        self.tree = tree;
        self.items = items;

        Ok(())
    }


    fn handle_mouse_event(
        &mut self,
        mouse_event: crossterm::event::MouseEvent,
        focus: Focus,
    ) -> Result<Action, Action> {
        match focus {
            Focus::Connections => match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    self.state.scroll_down(1);
                    Ok(Action::Nothing)
                }
                MouseEventKind::ScrollUp => {
                    self.state.scroll_up(1);
                    Ok(Action::Nothing)
                }

                MouseEventKind::Down(_button) => {
                    self.state
                        .click_at(Position::new(mouse_event.column, mouse_event.row));
                    Ok(Action::Nothing)
                }
                _ => Ok(Action::Nothing),
            },
            _ => Ok(Action::Nothing),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::Connections => {
                if [self.config.key_config.quit, self.config.key_config.exit]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.change_focus {
                    Ok(Action::ChangeFocus(Focus::Viewer))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_up();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_down();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_left,
                    self.config.key_config.arrow_left,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_left();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_right,
                    self.config.key_config.arrow_right,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_right();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.select_last {
                    self.state.select_last();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.select_first {
                    self.state.select_first();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.toggle_selected {
                    self.state.toggle_selected();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.activate {
                    let selected = self.state.selected().to_vec();
                    self.activate(selected)
                } else if key == self.config.key_config.list_item {
                    let selected = self.state.selected().to_vec();
                    if selected.len() == 2 {
                        // listing a cloud provider
                        self.list_cloud_provider(selected, Focus::Connections)
                    } else if selected.len() == 3 {
                        // listing a config
                        self.list_configuration(selected.clone())
                    } else {
                        Ok(Action::Nothing)
                    }
                } else if key == self.config.key_config.filter {
                    // activate filter
                    self.filter.switch_active_status();
                    Ok(Action::ChangeFocus(Focus::ConnectionsFilter))
                } else {
                    Ok(Action::Nothing)
                }
            }
            Focus::ConnectionsFilter => {
                let action = self.filter.handle_key_event(key_event, focus)?;
                match action {
                    Action::Filter(txt) => {
                        let tree_items = self
                            .tree
                            .nodes()
                            .filter(|n| !n.has_children())
                            .map(|n| n.value().to_string())
                            .collect();

                        self.filter.engage_filter(txt, tree_items)
                    }
                    _ => Ok(action),
                }
            }
            Focus::ConnectionFilterResults => self
                .filter
                .filter_results_handle_key_event(key_event, focus),
            _ => Ok(Action::Nothing),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: Focus) -> Result<(), String> {
        let focused = matches!(focus, Focus::Connections);
        let [content, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let [connections, _viewer] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        match Tree::new(&self.items) {
            Ok(tree) => {
                let widget =
                    tree.block(Block::bordered().title("Cloud Connections").border_style(
                        if focused {
                            Style::new().blue()
                        } else {
                            Style::default()
                        },
                    ))
                    .highlight_style(if focused {
                        Style::new()
                            .fg(Color::Black)
                            .bg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    })
                    .experimental_scrollbar(Some(
                        Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                            .begin_symbol(None)
                            .track_symbol(None)
                            .end_symbol(None),
                    ));

                frame.render_widget(Clear, connections);
                frame.render_stateful_widget(widget, connections, &mut self.state);
                self.filter.draw(frame, connections, focus)?;
                Ok(())
            }
            Err(_) => Err("all item identifiers need to be unique in connection tree".to_string()),
        }
    }

    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String> {
        self.config = config;
        self.filter.register_config(self.config.clone(), focus)
    }
}

impl TreeComponent for Connections {
    fn get_tree(&mut self) -> ETree<String> {
        self.tree.clone()
    }

    fn select_item(&mut self, selection: &str, focus: Focus) -> Result<(), String> {
        if matches!(focus, Focus::Connections) {
            let mut tree_item_path: Vec<String> = vec![];

            self.create_tree_item_path(&mut tree_item_path, Some(selection));

            tree_item_path.reverse();
            self.filter.switch_active_status();

            self.state.select(tree_item_path);
        }
        Ok(())
    }
}
