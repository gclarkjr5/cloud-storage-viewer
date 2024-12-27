use ego_tree::{NodeId, NodeRef, Tree};
use std::io::BufRead;
use std::process::Command;
use tui_tree_widget::{TreeItem, TreeState};

use crate::app::Cloud;
use crate::config::GcloudConfig;

#[derive(Debug)]
pub struct Connections {
    pub active: Option<String>,
    pub state: TreeState<String>,
    pub tree: Tree<String>,
    pub items: Vec<TreeItem<'static, String>>,
}

impl Connections {
    pub fn new() -> Self {
        let active = GcloudConfig::get_active_config();

        let clouds = [Cloud::Azure, Cloud::Gcs, Cloud::S3];
        let mut tree = Tree::new("Connections".to_string());
        let mut items = vec![];

        clouds.iter().for_each(|cloud| {
            tree.root_mut().append(cloud.to_string());
        });

        tree.nodes()
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                    .expect("error creating nodes under parent");

                add_children(node, &mut ti, 0);
                items.push(ti);
            });

        // let items = make_items(tree.clone(), 0);
        Self {
            active,
            state: TreeState::default(),
            tree,
            items,
        }
    }

    pub fn make_items(
        &mut self,
        tree: Tree<String>,
        page_idx: usize,
    ) -> Vec<TreeItem<'static, String>> {
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

    pub fn list_items(&mut self, path: Vec<String>) -> Option<()> {
        // find the node to append to
        let found_node = find_node_to_append(self.tree.clone(), path);

        // if empty dont do anything
        found_node.as_ref()?;
        // if found_node.is_none() {
        //     return None;
        // }
        let (selected, node_to_append_to) = found_node.unwrap();

        // get gcloud config path
        let gcloud_configuration_dir = dirs::home_dir()
            .unwrap()
            .join(".config/gcloud/configurations");

        // if at cloud level, list children which are essentially cli configs for that cloud
        match selected.as_str() {
            "Azure Data Lake Storage" => None,
            "Google Cloud Storage" => {
                // list available configs
                let configs = cli_command("ls", vec![gcloud_configuration_dir.to_str().unwrap()]);

                // style config names, and add to tree
                configs.lines().for_each(|listing| {
                    let res = listing.expect("error getting listing from stdout");

                    let mut res_chars = res.chars();
                    for _ in 0..7 {
                        res_chars.next();
                    }

                    let res_cleaned = res_chars.as_str().to_string();

                    self.tree
                        .get_mut(node_to_append_to)
                        .expect("error getting mutable node")
                        .append(res_cleaned);
                });
                // convert tree into tree widget items
                self.items = self.make_items(self.tree.clone(), 0);
                None
            }
            "AWS S3" => None,
            "Connections" => None,
            _ => {
                if selected != self.active.clone().unwrap() {
                    None
                } else {
                    Some(())
                }
            }
        }
    }
}

fn add_children(node: NodeRef<String>, tree_item: &mut TreeItem<String>, page_idx: usize) {
    if node.has_children() {
        let num_node_children = node.children().count();

        // let results_pager = ResultsPager::new(
        //     20,
        //     page_idx,
        //     num_node_children,
        //     tree_item.identifier().clone(),
        // );

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

fn cli_command(program: &str, args: Vec<&str>) -> Vec<u8> {
    Command::new(program)
        .args(args)
        .output()
        .expect("error processing command")
        .stdout
}
