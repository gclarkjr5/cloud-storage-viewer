use crate::app::make_items;
use ego_tree::Tree;
use tui_tree_widget::{TreeItem, TreeState};

pub struct Viewer {
    pub state: TreeState<String>,
    pub tree: Tree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub results_page_idx: usize,
}

impl Viewer {
    pub fn new() -> Self {
        let tree = Tree::new("CloudFS".to_string());
        let items = make_items(tree.clone(), 0);

        Self {
            state: TreeState::default(),
            tree,
            items,
            results_page_idx: 0,
        }
    }
}
