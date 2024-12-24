use ego_tree::{NodeId, NodeRef, Tree};
use std::process::Command;
use std::{fmt, io::BufRead};
use tui_tree_widget::{TreeItem, TreeState};

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

// trait CloudStorageViewer {
//     // fn list_connections();
//     // fn activate_connection();
//     fn list_items(&mut self, path_identifier: Option<Vec<String>>);
// }

// impl CloudStorageViewer for Cloud {
//     pub fn list_items(&mut self, path_identifier: Option<Vec<String>>) {
//         match self {
//             Cloud::Azure => (),
//             Cloud::Gcs => list_gcs_items,
//             Cloud::S3 => (),
//         }
//     }
// }

#[derive(Debug)]
pub enum CurrentScreen {
    Connections,
    Viewer,
}

#[derive(Debug, Clone)]
pub struct GcloudConfigs {
    name: String,
    is_active: String,
    account: String,
    project: String,
}

impl GcloudConfigs {
    pub fn get_configs() -> Vec<Self> {
        Command::new("gcloud")
            .args(vec!["config", "configurations", "list"])
            .output()
            .expect("error getting config list")
            .stdout
            .lines()
            .skip(1)
            .map(|line| {
                let splits = line
                    .expect("error getting line in config list")
                    .split_whitespace()
                    .map(|split| split.to_string())
                    .collect::<Vec<String>>();

                Self {
                    name: splits[0].clone(),
                    is_active: splits[1].clone(),
                    account: splits[2].clone(),
                    project: splits[3].clone(),
                }
            })
            .collect::<Vec<Self>>()
    }

    pub fn get_active_config() -> Option<String> {
        let active_config = GcloudConfigs::get_configs()
            .iter()
            .find(|config| config.is_active == "True")
            .expect("error finding active account")
            .name
            .clone();
        Some(active_config)
    }
}

#[must_use]
pub struct App {
    pub current_screen: CurrentScreen,
    // pub cloud: Cloud,
    pub viewer_state: TreeState<String>,
    pub viewer_tree: Tree<String>,
    pub viewer_items: Vec<TreeItem<'static, String>>,
    pub active_connection: Option<String>,
    pub connection_state: TreeState<String>,
    pub connection_tree: Tree<String>,
    pub connection_items: Vec<TreeItem<'static, String>>,
}

impl App {
    pub fn new() -> Self {
        let clouds = [Cloud::Azure, Cloud::Gcs, Cloud::S3];
        let mut connection_tree = Tree::new("Connections".to_string());

        clouds.iter().for_each(|cloud| {
            connection_tree.root_mut().append(cloud.to_string());
        });
        let connection_items = make_items(connection_tree.clone());

        let viewer_tree = Tree::new("CloudFS".to_string());
        // let viewer_tree = None;
        let viewer_items = make_items(viewer_tree.clone());

        let active_connection = GcloudConfigs::get_active_config();

        Self {
            current_screen: CurrentScreen::Connections,
            // cloud: Cloud::Gcs,
            viewer_state: TreeState::default(),
            viewer_tree,
            viewer_items,
            active_connection,
            connection_state: TreeState::default(),
            connection_tree,
            connection_items,
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
        if Some(new_config_name.clone()) == self.active_connection {
            return true;
        }

        // change active connection
        // empty the viewer
        self.viewer_tree = Tree::new("CloudFS".to_string());
        self.viewer_items = make_items(self.viewer_tree.clone());

        Command::new("gcloud")
            .args(["config", "configurations", "activate"])
            .arg(new_config_name.clone())
            .output()
            .expect("error creating new gcloud config");

        self.active_connection = GcloudConfigs::get_active_config();
        true
    }
    // pub fn new_config(self, new_config_name: &str) {
    //     Command::new("gcloud")
    //         .args(["config", "configurations", "create"])
    //         .arg(new_config_name)
    //         .output()
    //         .expect("error creating new gcloud config");
    // }

    pub fn list_items(&mut self, path_identifier: Option<Vec<String>>) {
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
                    let found_node = find_node_to_append(self.connection_tree.clone(), path);

                    if found_node.is_none() {
                        return {};
                    }
                    let (selected, node_to_append_to) = found_node.unwrap();
                    match selected.as_str() {
                        "Azure Data Lake Storage" => {}
                        "Google Cloud Storage" => {
                            let configs = cli_command(
                                "ls",
                                vec!["/Users/6148139/.config/gcloud/configurations"],
                            );
                            // let (selected, node_to_append_to) =
                            //     find_node_to_append(self.connection_tree.clone(), path);

                            configs.lines().for_each(|listing| {
                                let res = listing.expect("error getting listing from stdout");

                                let mut res_chars = res.chars();
                                for _ in 0..7 {
                                    res_chars.next();
                                }

                                let res_cleaned = res_chars.as_str().to_string();

                                self.connection_tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res_cleaned);
                            });
                            self.connection_items = make_items(self.connection_tree.clone());
                        }
                        "AWS S3" => {}
                        "Connections" => {}
                        _ => {
                            let found_node = find_node_to_append(
                                self.viewer_tree.clone(),
                                vec!["CloudFS".to_string()],
                            );
                            if found_node.is_none() {
                                return {};
                            }
                            let (_, node_to_append_to) = found_node.unwrap();
                            let buckets = cli_command("gsutil", vec!["ls"]);
                            buckets.lines().for_each(|listing| {
                                let res = listing.expect("error getting listing from stdout");

                                self.viewer_tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res);
                            });
                            self.viewer_items = make_items(self.viewer_tree.clone());
                            self.current_screen = CurrentScreen::Viewer
                        }
                    }
                }
                CurrentScreen::Viewer => {
                    let found_node = find_node_to_append(self.viewer_tree.clone(), path.clone());
                    if found_node.is_none() {
                        return {};
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
                                self.viewer_tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res);
                            });
                            self.viewer_items = make_items(self.viewer_tree.clone());
                        }
                        false => {}
                    }
                }
            },
        }
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

    let node = tree
        .nodes()
        .find(|node| node.value() == selected)
        .expect("error finding node");

    if node.has_children() {
        return None;
    }

    Some((selected.to_string(), node.id()))
}

fn add_children(node: NodeRef<String>, tree_item: &mut TreeItem<String>) {
    if node.has_children() {
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
            add_children(n, &mut child_ti);
            tree_item
                .add_child(child_ti)
                .expect("error adding child to the tree item");
        })
    }
}

fn make_items(tree: Tree<String>) -> Vec<TreeItem<'static, String>> {
    let nodes = tree.nodes();
    let mut root_vec = vec![];

    nodes
        .filter(|node| node.parent().is_none())
        .for_each(|node| {
            let val = node.value().to_string();
            let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                .expect("error creating nodes under parent");

            add_children(node, &mut ti);
            root_vec.push(ti);
        });

    root_vec
}
