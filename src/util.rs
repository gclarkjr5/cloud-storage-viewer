use ego_tree::{iter::Nodes, NodeRef};
use tui_tree_widget::TreeItem;

use crate::{app::Focus, components::results_pager::ResultsPager};

pub fn make_tree_items(
    nodes: Nodes<String>,
    results_pager: &mut ResultsPager,
    focus: Focus,
) -> Vec<TreeItem<'static, String>> {
    let mut root_vec = vec![];

    nodes
        .filter(|node| node.parent().is_none())
        .for_each(|node| {
            let identifier = node.value().to_string();
            let mut ti = TreeItem::new(identifier.clone(), identifier.clone(), vec![])
                .expect("error creating nodes under parent");

            add_children(node, &mut ti, &mut results_pager.clone(), focus);
            root_vec.push(ti);
        });

    root_vec
}

pub fn add_children(
    node: NodeRef<String>,
    tree_item: &mut TreeItem<String>,
    results_pager: &mut ResultsPager,
    focus: Focus,
) {
    if node.has_children() {
        let num_node_children = node.children().count();

        // // if there are more children than the allowed results per page, page the results
        if num_node_children > results_pager.results_per_page {
            // collect children into a vec of vecs of chunk size specified in pager
            let node_children_vec: Vec<NodeRef<String>> = node.children().collect();
            let node_children_pages: Vec<Vec<NodeRef<String>>> = node_children_vec
                .chunks(results_pager.results_per_page)
                .map(|chunk| chunk.to_vec())
                .collect();

            // save number of pages
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
                        let clean_text = match focus {
                            Focus::Connections => {
                                let split_text = child_val.split('/');
                                split_text.last().unwrap().to_string()
                            }
                            Focus::Viewer => {
                                let split_text = child_val.split('/');
                                if split_text.clone().count() <= 4 {
                                    child_val.clone()
                                } else if split_text.clone().last().unwrap() == "" {
                                    split_text.rev().nth(1).unwrap().to_string() + "/"
                                } else {
                                    split_text.last().unwrap().to_string()
                                }
                            }
                            _ => child_val.clone(),
                        };

                        let mut child_ti =
                            TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
                                .expect("error creating child node");

                        add_children(*n, &mut child_ti, &mut results_pager.clone(), focus);
                        tree_item
                            .add_child(child_ti)
                            .expect("error adding child to the tree item");
                    });
            }
        } else {
            node.children().for_each(|n| {
                let child_val = n.value().to_string();
                let clean_text = match focus {
                    Focus::Connections => {
                        let split_text = child_val.split('/');
                        split_text.last().unwrap().to_string()
                    }
                    Focus::Viewer => {
                        let split_text = child_val.split('/');
                        if split_text.clone().count() <= 4 {
                            child_val.clone()
                        } else if split_text.clone().last().unwrap() == "" {
                            split_text.rev().nth(1).unwrap().to_string() + "/"
                        } else {
                            split_text.last().unwrap().to_string()
                        }
                    }
                    _ => child_val.clone(),
                };

                let mut child_ti = TreeItem::new(child_val.clone(), clean_text.clone(), vec![])
                    .expect("error creating child node");
                add_children(n, &mut child_ti, &mut results_pager.clone(), focus);
                tree_item
                    .add_child(child_ti)
                    .expect("error adding child to the tree item");
            });
        }
    }
}
