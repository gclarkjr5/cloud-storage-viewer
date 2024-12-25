use ego_tree::Tree;
use tui_tree_widget::{TreeItem, TreeState};

use crate::app::{make_items, Cloud};
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

        clouds.iter().for_each(|cloud| {
            tree.root_mut().append(cloud.to_string());
        });

        let items = make_items(tree.clone(), 0);
        Self {
            active,
            state: TreeState::default(),
            tree,
            items,
        }
    }
}
