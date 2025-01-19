use super::results_pager::ResultsPager;
use super::Component;
use crossterm::event::MouseEventKind;
use ego_tree::{NodeId, Tree as ETree};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Clear, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use std::io::Result;
use std::process::Command;
use std::vec;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::action::Action;
use crate::app::Focus;
use crate::config::cloud_config::CloudProvider;
use crate::config::Config;
use crate::key::Key;
use crate::util;

#[derive(Debug)]
pub struct Connections {
    pub state: TreeState<String>,
    pub tree: ETree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub config: Config,
    pub results_pager: ResultsPager,
}

impl Default for Connections {
    fn default() -> Self {
        Self::new()
    }
}

impl Connections {
    pub fn new() -> Self {
        Self {
            state: TreeState::default(),
            tree: ETree::new(String::new()),
            items: Vec::new(),
            config: Config::default(),
            results_pager: ResultsPager::default(),
        }
    }

    pub fn list_cloud_provider(&mut self, selection: Vec<String>) -> Result<()> {
        let cloud_provider: CloudProvider = selection[1].clone().into();
        // find the node to append to
        let found_node = self.find_node_to_append(cloud_provider.clone().into());

        // if empty dont do anything
        if found_node.is_none() {
            return Ok(());
        }
        let (_selected, node_to_append_to) = found_node.unwrap();

        self.config
            .cloud_config
            .cloud_providers
            .iter()
            .for_each(|cp| match (cp, &cloud_provider) {
                (CloudProvider::Azure(_), CloudProvider::Azure(_)) => (),
                (CloudProvider::Gcs(configs), CloudProvider::Gcs(_)) => {
                    configs.iter().for_each(|config| {
                        let res = config.name.clone();

                        self.tree
                            .get_mut(node_to_append_to)
                            .expect("error getting mutable node")
                            .append(res);
                    })
                }
                (CloudProvider::S3(_), CloudProvider::S3(_)) => (),
                _ => (),
            });

        // convert tree into tree widget items
        self.items = util::make_tree_items(self.tree.nodes(), &mut self.results_pager);
        self.state.open(selection);
        Ok(())
    }

    pub fn list_configuration(&mut self, path: Vec<String>) -> Result<Vec<u8>> {
        // set active cloud
        self.config.cloud_config.activate_config(path)?;

        let output = cli_command("gsutil", vec!["ls"]);

        Ok(output)
    }

    pub fn find_node_to_append(&mut self, path_identifier: String) -> Option<(String, NodeId)> {
        let found_node = self
            .tree
            .nodes()
            .find(|node| node.value() == &path_identifier);

        found_node?;

        let node = found_node.expect("error unwrapping found node");

        if node.has_children() {
            return None;
        }

        Some((path_identifier, node.id()))
    }
}

impl Component for Connections {
    fn init(&mut self) -> Result<()> {
        let mut tree = ETree::new("Connections".to_string());

        self.config
            .cloud_config
            .cloud_providers
            .iter()
            .for_each(|cloud| {
                tree.root_mut().append(cloud.to_string());
            });

        let mut items = vec![];

        tree.nodes()
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                    .expect("error creating nodes under parent");

                let mut results_pager = self.results_pager.clone();

                util::add_children(node, &mut ti, &mut results_pager);
                items.push(ti);
            });

        self.tree = tree;
        self.items = items;

        Ok(())
    }

    fn handle_mouse_event(
        &mut self,
        mouse_event: crossterm::event::MouseEvent,
        focus: Focus,
    ) -> Result<Option<Action>> {
        match focus {
            Focus::Connections => match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    self.state.scroll_down(1);
                    Ok(Some(Action::Nothing))
                }
                MouseEventKind::ScrollUp => {
                    self.state.scroll_up(1);
                    Ok(Some(Action::Nothing))
                }
                MouseEventKind::Down(_button) => {
                    self.state
                        .click_at(Position::new(mouse_event.column, mouse_event.row));
                    Ok(Some(Action::Nothing))
                }
                _ => Ok(Some(Action::Nothing)),
            },
            _ => Ok(Some(Action::Nothing)),
        }
    }

    fn handle_key_event(&mut self, key: Key, focus: Focus) -> Result<Option<Action>> {
        match focus {
            Focus::Connections => {
                if [self.config.key_config.quit, self.config.key_config.exit]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    Ok(Some(Action::Quit))
                } else if key == self.config.key_config.change_focus {
                    Ok(Some(Action::ChangeFocus))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_up();
                    Ok(None)
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_down();
                    Ok(Some(Action::Nothing))
                } else if [
                    self.config.key_config.key_left,
                    self.config.key_config.arrow_left,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_left();
                    Ok(None)
                } else if [
                    self.config.key_config.key_right,
                    self.config.key_config.arrow_right,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_right();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.select_last {
                    self.state.select_last();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.select_first {
                    self.state.select_first();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.toggle_selected {
                    self.state.toggle_selected();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.activate_connection {
                    let selected = self.state.selected().to_vec();

                    // first is the root, second is the cloud provider
                    if selected.len() < 3 {
                        Ok(Some(Action::Nothing))
                    } else {
                        Ok(Some(Action::ActivateConfig(selected)))
                    }
                } else if key == self.config.key_config.list_item {
                    let selected = self.state.selected().to_vec();

                    if selected.len() == 2 {
                        // listing a cloud provider

                        let cloud_provider: CloudProvider = selected[1].clone().into();
                        self.config.cloud_config.list_config(cloud_provider.clone());
                        self.list_cloud_provider(selected)?;
                        Ok(Some(Action::ListCloudProvider(
                            self.config.cloud_config.clone(),
                        )))
                    } else if selected.len() == 3 {
                        // listing a config
                        let buckets = self.list_configuration(selected.clone())?;
                        Ok(Some(Action::ListConfiguration(
                            self.config.cloud_config.clone(),
                            vec![format!("{}", self.config.cloud_config)],
                            buckets,
                        )))
                    } else {
                        Ok(Some(Action::Nothing))
                    }
                } else {
                    Ok(Some(Action::Nothing))
                }
            }
            _ => Ok(Some(Action::Nothing)),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: Focus) -> Result<()> {
        let focused = matches!(focus, Focus::Connections);
        let [content, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let [connections, _] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        let widget = Tree::new(&self.items)
            .expect("all item identifieers are unique")
            .block(
                Block::bordered()
                    .title("Cloud Connections")
                    .border_style(if focused {
                        Style::new().blue()
                    } else {
                        Style::default()
                    }),
            )
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
        Ok(())
    }

    fn register_config(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

// fn add_children(node: NodeRef<String>, tree_item: &mut TreeItem<String>, page_idx: usize) {
//     if node.has_children() {
//         let num_node_children = node.children().count();

//         if num_node_children > 20 {
//             let node_children_vec: Vec<NodeRef<String>> = node.children().collect();
//             let node_children_pages: Vec<Vec<NodeRef<String>>> = node_children_vec
//                 .chunks(20)
//                 .map(|chunk| chunk.to_vec())
//                 .collect();

//             let num_pages = node_children_pages.len();

//             let page_of_children = node_children_pages[page_idx].clone();

//             page_of_children
//                 .iter()
//                 .enumerate()
//                 .for_each(|(child_idx, n)| {
//                     let child_val = n.value().to_string();
//                     let split_text = child_val.split('/');

//                     let clean_text = if split_text.clone().count() <= 4 {
//                         child_val.clone()
//                     } else if split_text.clone().last().unwrap() == "" {
//                         split_text.rev().nth(1).unwrap().to_string() + "/"
//                     } else {
//                         split_text.last().unwrap().to_string()
//                     };

//                     let mut child_ti = TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
//                         .expect("error creating child node");

//                     add_children(*n, &mut child_ti, page_idx);
//                     tree_item
//                         .add_child(child_ti)
//                         .expect("error adding child to the tree item");

//                     // if not last page, add ... Press 'L' for next page ...
//                     if page_idx + 1 < num_pages {
//                         // not on last page yet
//                         if child_idx + 1 == 20 {
//                             // add delimiter

//                             let next_page_text = "... Press 'L' for next page ...".to_string();
//                             let delim_ti =
//                                 TreeItem::new(next_page_text.clone(), next_page_text, vec![])
//                                     .expect("error creating child node");

//                             tree_item
//                                 .add_child(delim_ti)
//                                 .expect("error adding delimiter to the tree item");
//                         }
//                     } else {
//                         {}
//                     }
//                 })
//         } else {
//             node.children().for_each(|n| {
//                 let child_val = n.value().to_string();
//                 let split_text = child_val.split('/');

//                 let clean_text = if split_text.clone().count() <= 4 {
//                     child_val.clone()
//                 } else if split_text.clone().last().unwrap() == "" {
//                     split_text.rev().nth(1).unwrap().to_string() + "/"
//                 } else {
//                     split_text.last().unwrap().to_string()
//                 };

//                 let mut child_ti = TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
//                     .expect("error creating child node");
//                 add_children(n, &mut child_ti, page_idx);
//                 tree_item
//                     .add_child(child_ti)
//                     .expect("error adding child to the tree item");
//             })
//         }
//     }
// }

fn cli_command(program: &str, args: Vec<&str>) -> Vec<u8> {
    Command::new(program)
        .args(args)
        .output()
        .expect("error processing command")
        .stdout
}
