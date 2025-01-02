use std::{io::BufRead, process::Command};

use ego_tree::{NodeId, NodeRef, Tree};
use tui_tree_widget::{TreeItem, TreeState};

#[derive(Debug, Clone)]
pub struct ResultsPager {
    pub results_per_page: usize,
    pub page_idx: usize,
    pub num_pages: usize,
    pub total_results: usize,
    pub paged_item: String,
    pub remainder: usize,
}

impl ResultsPager {
    pub fn default() -> Self {
        Self {
            results_per_page: 20,
            page_idx: 0,
            num_pages: 1,
            total_results: 0,
            paged_item: "CloudFS".to_string(),
            remainder: 0,
        }
    }

    pub fn init(&mut self, results: &Vec<u8>, selection: &str) {
        let num_results = results.lines().count();

        self.total_results = num_results;
        self.paged_item = selection.to_string();
        match (
            num_results / self.results_per_page,
            num_results % self.results_per_page,
        ) {
            (div, rem) if div < 1 => {
                self.num_pages = 1;
                self.remainder = rem;
            }
            (div, rem) if rem > 0 => {
                self.num_pages = div + 1;
                self.remainder = rem;
            }
            (div, rem) => {
                self.num_pages = div;
                self.remainder = rem;
            }
        }
    }
}

pub struct Viewer {
    pub state: TreeState<String>,
    pub tree: Tree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub results_pager: ResultsPager,
}

impl Viewer {
    pub fn new(active_connection: &str) -> Self {
        let tree = Tree::new(active_connection.to_string());
        let nodes = tree.nodes();
        let mut items = vec![];
        let results_pager = ResultsPager::default();

        nodes
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                    .expect("error creating nodes under parent");

                add_children(node, &mut ti, &mut results_pager.clone());
                items.push(ti);
            });

        Self {
            state: TreeState::default(),
            tree,
            items,
            results_pager,
        }
    }

    pub fn increase_results_page(&mut self) -> Option<()> {
        // only increase page idx if we are on a page less than the number of pages
        if self.results_pager.page_idx + 1 < self.results_pager.num_pages {
            self.results_pager.page_idx += 1;
            Some(())
        } else {
            None
        }
    }

    // pub fn next_page(&mut self) -> bool {
    //     true
    // }

    // pub fn previous_page(&mut self) -> bool {
    //     true
    // }

    pub fn decrease_results_page(&mut self) -> Option<()> {
        // only decrease page idx if we are on a page higher than 1
        if self.results_pager.page_idx + 1 > 1 {
            self.results_pager.page_idx -= 1;
            Some(())
        } else {
            None
        }
    }

    // pub fn refresh_items(&mut self, path: Vec<String>) {
    //     let (_, found_node) = self.find_node_to_append(path.clone()).unwrap();

    //     // HOW DO WE REMOVE CHILDREN FROM A NODE
    //     // self.tree
    //     //     .get_mut(node_id)
    //     //     .expect("error getting mutable node")
    //     //     .detach();
    //     self.items = self.make_items(self.tree.clone(), self.results_pager.page_idx);
    // }

    pub fn make_items(&mut self) -> Vec<TreeItem<'static, String>> {
        let nodes = self.tree.nodes();
        let mut root_vec = vec![];

        nodes
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                    .expect("error creating nodes under parent");

                let mut results_pager = self.results_pager.clone();

                add_children(node, &mut ti, &mut results_pager);
                root_vec.push(ti);
            });

        root_vec
    }

    pub fn list_items(&mut self, path: Vec<String>, action: &str) -> Option<()> {
        // find node, verify, unwrap, and set pager
        let found_node = self.find_node_to_append(path.clone(), action);
        found_node.as_ref()?;
        let (view_selection, node_to_append_to) = found_node.unwrap();

        // is the selection a directory?
        let is_directory = view_selection
            .chars()
            .last()
            .expect("error getting last char")
            == '/';

        match is_directory {
            true => {
                // if so
                // list it
                // self.results_pager.paged_item = view_selection.clone();
                let output = self.cli_command("gsutil", vec!["ls", view_selection.as_str()]);

                // set the pager
                self.results_pager.init(&output, &view_selection);

                // add nodes to tree
                output.lines().for_each(|listing| {
                    let res = listing.expect("error getting listing from stdout");
                    self.tree
                        .get_mut(node_to_append_to)
                        .expect("error getting mutable node")
                        .append(res);
                });

                // remake tree widget
                self.items = self.make_items();
                None
            }
            false => {
                // coming from listing the root connection, making buckets
                let output = self.cli_command("gsutil", vec!["ls"]);

                // set the pager
                self.results_pager
                    .init(&output, &self.results_pager.paged_item.clone());

                // add nodes to tree
                output.lines().for_each(|listing| {
                    let res = listing.expect("error getting listing from stdout");
                    self.tree
                        .get_mut(node_to_append_to)
                        .expect("error getting mutable node")
                        .append(res);
                });

                // remake tree widget
                self.items = self.make_items();

                None
            }
        }
    }

    pub fn find_node_to_append(
        &mut self,
        path_identifier: Vec<String>,
        action: &str,
    ) -> Option<(String, NodeId)> {
        // take path identifier and grab last item
        let selected = path_identifier
            .iter()
            .last()
            .expect("error getting selected item")
            .as_str();

        // use the selction to find the node in the tree
        let found_node = self.tree.nodes().find(|node| node.value() == selected);

        found_node.as_ref()?;
        let node = found_node.expect("error unwrapping found node");

        // if node already has children, we wont don anything
        // match action {
        //     "request" => {
        //         if node.has_children() {
        //             return None;
        //         }
        //     }
        // }
        if node.has_children() && action == "request" {
            return None;
        }

        // return the selction and the node id
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

fn add_children(
    node: NodeRef<String>,
    tree_item: &mut TreeItem<String>,
    results_pager: &mut ResultsPager,
) {
    if node.has_children() {
        let num_node_children = node.children().count();

        // if there are more children than the allowed results per page, page the results
        if num_node_children > results_pager.results_per_page {
            // collect children into a vec of vecs of specified chunk size
            let node_children_vec: Vec<NodeRef<String>> = node.children().collect();
            let node_children_pages: Vec<Vec<NodeRef<String>>> = node_children_vec
                .chunks(results_pager.results_per_page)
                .map(|chunk| chunk.to_vec())
                .collect();

            // gather number of pages
            results_pager.num_pages = node_children_pages.len();

            // while current page is not the last,
            if results_pager.page_idx < results_pager.num_pages {
                // only get the inner vec of children of the current page index
                let page_of_children = node_children_pages[results_pager.page_idx].clone();

                // for each child in this inner vec, we will create tree items
                page_of_children
                    .iter()
                    // .enumerate()
                    .for_each(|n| {
                        // prettify child text
                        let child_val = n.value().to_string();
                        let split_text = child_val.split('/');
                        let clean_text = if split_text.clone().count() <= 4 {
                            child_val.clone()
                        } else if split_text.clone().last().unwrap() == "" {
                            split_text.rev().nth(1).unwrap().to_string() + "/"
                        } else {
                            split_text.last().unwrap().to_string()
                        };

                        let mut child_ti =
                            TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
                                .expect("error creating child node");

                        add_children(*n, &mut child_ti, &mut results_pager.clone());
                        tree_item
                            .add_child(child_ti)
                            .expect("error adding child to the tree item");
                    });
            }
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
                add_children(n, &mut child_ti, &mut results_pager.clone());
                tree_item
                    .add_child(child_ti)
                    .expect("error adding child to the tree item");
            });
        }
    }
}
