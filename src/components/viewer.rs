use std::{io::BufRead, process::Command};

// use crate::app::make_items;
use ego_tree::{NodeId, NodeRef, Tree};
use tui_tree_widget::{TreeItem, TreeState};

pub struct ResultsPager {
    pub results_per_page: usize,
    pub page_idx: usize,
    pub total_results: usize,
    pub paged_item: String,
}

impl ResultsPager {
    pub fn default() -> Self {
        Self {
            results_per_page: 20,
            page_idx: 0,
            total_results: 0,
            paged_item: "CloudFS".to_string(),
        }
    }
    pub fn new(
        &mut self,
        results_per_page: usize,
        page_idx: usize,
        total_results: usize,
        paged_item: String,
    ) {
        self.results_per_page = results_per_page;
        self.page_idx = page_idx;
        self.total_results = total_results;
        self.paged_item = paged_item;
    }
}

pub struct Viewer {
    pub state: TreeState<String>,
    pub tree: Tree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub results_pager: ResultsPager,
    pub results_page_idx: usize,
}

impl Viewer {
    pub fn new() -> Self {
        let tree = Tree::new("CloudFS".to_string());
        // let items = self.make_items(tree.clone(), 0);
        let results_pager = ResultsPager::default();

        let nodes = tree.nodes();
        let mut items = vec![];
        // let mut root_vec = vec![];

        nodes
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                    .expect("error creating nodes under parent");

                add_children(node, &mut ti, 0);
                items.push(ti);
            });

        Self {
            state: TreeState::default(),
            tree,
            items,
            results_pager,
            results_page_idx: 0,
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
        let found_node = self.find_node_to_append(self.tree.clone(), path.clone());

        found_node.as_ref()?;

        let (view_selection, node_to_append_to) = found_node.unwrap();
        let is_directory = view_selection
            .chars()
            .last()
            .expect("error getting last char")
            == '/';

        match is_directory {
            true => {
                let output = self.cli_command("gsutil", vec!["ls", view_selection.as_str()]);

                output.lines().for_each(|listing| {
                    let res = listing.expect("error getting listing from stdout");
                    self.tree
                        .get_mut(node_to_append_to)
                        .expect("error getting mutable node")
                        .append(res);
                });
                self.items = self.make_items(self.tree.clone(), self.results_page_idx);
                None
            }
            false => None,
        }
    }

    pub fn find_node_to_append(
        &mut self,
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

    pub fn cli_command(&mut self, program: &str, args: Vec<&str>) -> Vec<u8> {
        Command::new(program)
            .args(args)
            .output()
            .expect("error processing command")
            .stdout
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
