use std::process::Command;
use std::{fmt, io::BufRead};

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
pub enum Focus {
    Connections,
    Viewer,
}

#[must_use]
pub struct App {
    pub focus: Focus,
    // pub cloud: Cloud,
    pub connections: Connections,
    pub viewer: Viewer,
}

impl App {
    pub fn new() -> Self {
        Self {
            focus: Focus::Connections,
            // cloud: Cloud::Gcs,
            viewer: Viewer::new(),
            connections: Connections::new(),
        }
    }

    pub fn toggle_screen(&mut self) {
        match self.focus {
            Focus::Connections => self.focus = Focus::Viewer,
            Focus::Viewer => self.focus = Focus::Connections,
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
        self.viewer = Viewer::new();

        Command::new("gcloud")
            .args(["config", "configurations", "activate"])
            .arg(new_config_name.clone())
            .output()
            .expect("error creating new gcloud config");

        self.connections.active = GcloudConfig::get_active_config();
        true
    }

    pub fn increase_results_page(&mut self) -> Option<()> {
        if self.viewer.results_pager.page_idx + 1 < self.viewer.results_pager.num_pages {
            self.viewer.results_pager.page_idx += 1;
            Some(())
        } else {
            None
        }
    }

    pub fn list_items(&mut self, path_identifier: Option<Vec<String>>, page_idx: usize) -> bool {
        match path_identifier {
            None => {}

            Some(path) => match self.focus {
                Focus::Connections => {
                    let items = self.connections.list_items(path);

                    match items {
                        None => false,
                        Some(_) => {
                            // listings from configs will be buckets, which should append to the root node in the viewer
                            let found_node =
                                self.viewer.find_node_to_append(vec!["CloudFS".to_string()]);
                            if found_node.is_none() {
                                return false;
                            }
                            let (_, node_to_append_to) = found_node.unwrap();

                            // get the buckets
                            let buckets = self.viewer.cli_command("gsutil", vec!["ls"]);
                            buckets.lines().for_each(|listing| {
                                let res = listing.expect("error getting listing from stdout");

                                self.viewer
                                    .tree
                                    .get_mut(node_to_append_to)
                                    .expect("error getting mutable node")
                                    .append(res);
                            });
                            self.viewer.items =
                                self.viewer.make_items(self.viewer.tree.clone(), page_idx);
                            self.focus = Focus::Viewer;
                            true
                        }
                    };
                }
                Focus::Viewer => {
                    self.viewer.list_items(path, page_idx);
                }
            },
        }
        true
    }
}

// fn cli_command(program: &str, args: Vec<&str>) -> Vec<u8> {
//     Command::new(program)
//         .args(args)
//         .output()
//         .expect("error processing command")
//         .stdout
// }

// fn find_node_to_append(
//     tree: Tree<String>,
//     path_identifier: Vec<String>,
// ) -> Option<(String, NodeId)> {
//     let selected = path_identifier
//         .iter()
//         .last()
//         .expect("error getting selected item")
//         .as_str();

//     let found_node = tree.nodes().find(|node| node.value() == selected);
//     // .expect("error finding node");

//     if found_node.is_none() {
//         return None;
//     };

//     let node = found_node.expect("error unwrapping found node");

//     if node.has_children() {
//         return None;
//     }

//     Some((selected.to_string(), node.id()))
// }

// fn add_children(node: NodeRef<String>, tree_item: &mut TreeItem<String>, page_idx: usize) {
//     if node.has_children() {
//         let num_node_children = node.children().count();

//         let results_pager = ResultsPager::new(
//             20,
//             page_idx,
//             num_node_children,
//             tree_item.identifier().clone(),
//         );

//         if num_node_children > 20 {
//             let node_children_vec: Vec<NodeRef<String>> = node.children().collect();
//             let node_children_pages: Vec<Vec<NodeRef<String>>> = node_children_vec
//                 .chunks(20)
//                 .map(|chunk| chunk.to_vec())
//                 .collect();

//             let num_pages = node_children_pages.iter().count();

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

// pub fn make_items(tree: Tree<String>, page_idx: usize) -> Vec<TreeItem<'static, String>> {
//     let nodes = tree.nodes();
//     let mut root_vec = vec![];

//     nodes
//         .filter(|node| node.parent().is_none())
//         .for_each(|node| {
//             let val = node.value().to_string();
//             let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
//                 .expect("error creating nodes under parent");

//             add_children(node, &mut ti, page_idx);
//             root_vec.push(ti);
//         });

//     root_vec
// }
