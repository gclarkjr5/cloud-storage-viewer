use ego_tree::{NodeId, NodeRef, Tree};
use std::process::Command;
use std::{fmt, io::BufRead};
use tui_tree_widget::TreeItem;

use super::components::connections::Connections;
use super::components::viewer::Viewer;
use super::config::GcloudConfig;

pub enum Cloud {
    Azure,
    Gcs,
    S3,
}

impl fmt::Display for Cloud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cloud::Gcs => write!(f, "Google Cloud Storage"),
            Cloud::Azure => write!(f, "Azure Data Lake Storage"),
            Cloud::S3 => write!(f, "AWS S3"),
        }
    }
}

#[derive(Debug)]
pub enum CurrentScreen {
    Connections,
    Viewer,
}

#[must_use]
pub struct App {
    pub current_screen: CurrentScreen,
    // pub cloud: Cloud,
    pub connections: Connections,
    pub viewer: Viewer,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: CurrentScreen::Connections,
            // cloud: Cloud::Gcs,
            viewer: Viewer::new(),
            connections: Connections::new(),
        }
    }

    pub fn toggle_screen(&mut self) {
        match self.current_screen {
            CurrentScreen::Connections => self.current_screen = CurrentScreen::Viewer,
            CurrentScreen::Viewer => self.current_screen = CurrentScreen::Connections,
        }
    }

    pub fn activate_connection(&mut self, path_identifier: Option<Vec<String>>) -> bool {
        let new_config_name = path_identifier
            .expect("error getting path ident")
            .last()
            .unwrap()
            .clone();

        // already on active connectinon
        if Some(new_config_name.clone()) == self.connections.active {
            return true;
        }

        // change active connection
        // empty the viewer
        self.viewer.tree = Tree::new("CloudFS".to_string());
        self.viewer.items = make_items(self.viewer.tree.clone(), self.viewer.results_page_idx);

        Command::new("gcloud")
            .args(["config", "configurations", "activate"])
            .arg(new_config_name.clone())
            .output()
            .expect("error creating new gcloud config");

        self.connections.active = GcloudConfig::get_active_config();
        true
    }
    // pub fn new_config(self, new_config_name: &str) {
    //     Command::new("gcloud")
    //         .args(["config", "configurations", "create"])
    //         .arg(new_config_name)
    //         .output()
    //         .expect("error creating new gcloud config");
    // }

    pub fn increase_results_page(&mut self) {
        self.viewer.results_page_idx += 1;
    }

    pub fn list_items(&mut self, path_identifier: Option<Vec<String>>) -> bool {
        match path_identifier {
            None => {
                unimplemented!()
            }

            Some(path) => match self.current_screen {
                CurrentScreen::Connections => {
                    // find available configs
                    // find the connection node to append the configs to
                    // append the configs
                    // extract listing
                    // maybe mutate it
                    // get mutable node out of tree and append
                    // make recursive tree out of nodes
                    // let (selected, node_to_append_to) =
                    let found_node = find_node_to_append(self.connections.tree.clone(), path);

                    if found_node.is_none() {
                        return false;
                    }
                    let (selected, node_to_append_to) = found_node.unwrap();

                    let gcloud_configuration_dir = dirs::home_dir()
                        .unwrap()
                        .join(".config/gcloud/configurations");

                    match selected.as_str() {
                        "Azure Data Lake Storage" => {}
                        "Google Cloud Storage" => {
                            let configs =
                                cli_command("ls", vec![gcloud_configuration_dir.to_str().unwrap()]);
                            // let (selected, node_to_append_to) =
                            //     find_node_to_append(self.connection_tree.clone(), path);

                            configs.lines().for_each(|listing| {
                                let res = listing.expect("error getting listing from stdout");

                                let mut res_chars = res.chars();
                                for _ in 0..7 {
                                    res_chars.next();
                                }

                                let res_cleaned = res_chars.as_str().to_string();

                                self.connections
                                    .tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res_cleaned);
                            });
                            self.connections.items = make_items(
                                self.connections.tree.clone(),
                                self.viewer.results_page_idx,
                            );
                        }
                        "AWS S3" => {}
                        "Connections" => {}
                        _ => {
                            let found_node = find_node_to_append(
                                self.viewer.tree.clone(),
                                vec!["CloudFS".to_string()],
                            );
                            if found_node.is_none() {
                                return false;
                            }
                            let (_, node_to_append_to) = found_node.unwrap();
                            let buckets = cli_command("gsutil", vec!["ls"]);
                            buckets.lines().for_each(|listing| {
                                let res = listing.expect("error getting listing from stdout");

                                self.viewer
                                    .tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res);
                            });
                            self.viewer.items =
                                make_items(self.viewer.tree.clone(), self.viewer.results_page_idx);
                            self.current_screen = CurrentScreen::Viewer
                        }
                    }
                }
                CurrentScreen::Viewer => {
                    let found_node = find_node_to_append(self.viewer.tree.clone(), path.clone());
                    if found_node.is_none() {
                        return false;
                    }
                    let (view_selection, node_to_append_to) = found_node.unwrap();
                    let is_directory = view_selection
                        .chars()
                        .last()
                        .expect("error getting last char")
                        == '/';

                    match is_directory {
                        true => {
                            let output = cli_command("gsutil", vec!["ls", view_selection.as_str()]);

                            output.lines().for_each(|listing| {
                                let res = listing.expect("error getting listing from stdout");
                                self.viewer
                                    .tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res);
                            });
                            self.viewer.items =
                                make_items(self.viewer.tree.clone(), self.viewer.results_page_idx);
                        }
                        false => {}
                    }
                }
            },
        }
        true
    }
}

fn cli_command(program: &str, args: Vec<&str>) -> Vec<u8> {
    Command::new(program)
        .args(args)
        .output()
        .expect("error processing command")
        .stdout
}

fn find_node_to_append(
    tree: Tree<String>,
    path_identifier: Vec<String>,
) -> Option<(String, NodeId)> {
    let selected = path_identifier
        .iter()
        .last()
        .expect("error getting selected item")
        .as_str();

    let found_node = tree.nodes().find(|node| node.value() == selected);
    // .expect("error finding node");

    if found_node.is_none() {
        return None;
    };

    let node = found_node.expect("error unwrapping found node");

    if node.has_children() {
        return None;
    }

    Some((selected.to_string(), node.id()))
}

fn add_children(node: NodeRef<String>, tree_item: &mut TreeItem<String>, page_idx: usize) {
    if node.has_children() {
        let num_node_children = node.children().count();

        if num_node_children > 20 {
            let node_children_vec: Vec<NodeRef<String>> = node.children().collect();
            let node_children_pages: Vec<Vec<NodeRef<String>>> = node_children_vec
                .chunks(20)
                .map(|chunk| chunk.to_vec())
                .collect();

            let num_pages = node_children_pages.iter().count();

            let page_of_children = node_children_pages[page_idx].clone();

            page_of_children
                .iter()
                .enumerate()
                .for_each(|(child_idx, n)| {
                    let child_val = n.value().to_string();
                    let split_text = child_val.split('/');

                    let clean_text = if split_text.clone().count() <= 4 {
                        child_val.clone()
                    } else if split_text.clone().last().unwrap() == "" {
                        split_text.rev().nth(1).unwrap().to_string() + "/"
                    } else {
                        split_text.last().unwrap().to_string()
                    };

                    let mut child_ti = TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
                        .expect("error creating child node");

                    add_children(*n, &mut child_ti, page_idx);
                    tree_item
                        .add_child(child_ti)
                        .expect("error adding child to the tree item");

                    // if not last page, add ... Press 'L' for next page ...
                    if page_idx + 1 < num_pages {
                        // not on last page yet
                        if child_idx + 1 == 20 {
                            // add delimiter

                            let next_page_text = "... Press 'L' for next page ...".to_string();
                            let delim_ti =
                                TreeItem::new(next_page_text.clone(), next_page_text, vec![])
                                    .expect("error creating child node");

                            tree_item
                                .add_child(delim_ti)
                                .expect("error adding delimiter to the tree item");
                        }
                    } else {
                        {}
                    }
                })
        } else {
            node.children().for_each(|n| {
                let child_val = n.value().to_string();
                let split_text = child_val.split('/');

                let clean_text = if split_text.clone().count() <= 4 {
                    child_val.clone()
                } else if split_text.clone().last().unwrap() == "" {
                    split_text.rev().nth(1).unwrap().to_string() + "/"
                } else {
                    split_text.last().unwrap().to_string()
                };

                let mut child_ti = TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
                    .expect("error creating child node");
                add_children(n, &mut child_ti, page_idx);
                tree_item
                    .add_child(child_ti)
                    .expect("error adding child to the tree item");
            })
        }
    }
}

pub fn make_items(tree: Tree<String>, page_idx: usize) -> Vec<TreeItem<'static, String>> {
    let nodes = tree.nodes();
    let mut root_vec = vec![];

    nodes
        .filter(|node| node.parent().is_none())
        .for_each(|node| {
            let val = node.value().to_string();
            let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                .expect("error creating nodes under parent");

            add_children(node, &mut ti, page_idx);
            root_vec.push(ti);
        });

    root_vec
}
