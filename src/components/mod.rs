use std::result::Result;
use std::any::Any;

use crate::{action::Action, app::Focus, config::Config};
use crossterm::event::{KeyEvent, MouseEvent};
use ego_tree::{NodeId, Tree};
use ratatui::{layout::Rect, Frame};

// pub mod connection_filter;
// pub mod connection_filter_results;
pub mod connections;
pub mod error;
pub mod filter;
pub mod filter_results;
pub mod footer;
pub mod results_pager;
pub mod viewer;
// pub mod viewer_filter;
// pub mod viewer_filter_results;

pub trait Component {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: Focus) -> Result<(), String>;
    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action> {
        let _key_event = key_event;
        let _foucs = focus;
        Ok(Action::Skip)
    }
    fn handle_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        focus: Focus,
    ) -> Result<Action, Action> {
        let _mouse_event = mouse_event;
        let _foucs = focus;
        Ok(Action::Nothing)
    }
    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String>;
    fn report_error(&mut self, message: String) -> Result<(), String> {
        let _message = message;
        Ok(())
    }
    fn as_any_mut(&mut self) -> &mut dyn Any; // Mutable version
}

pub trait TreeComponent {
    fn list_item(&mut self, data: Vec<u8>, path: Vec<String>, focus: Focus) -> Result<(), Action> {
        let _data = data;
        let _path = path;
        let _focus = focus;
        Ok(())
    }
    fn select_item(&mut self, item: &str, focus: Focus) -> Result<(), String> {
        let _item = item;
        let _focus = focus;
        Ok(())
    }
    fn find_node_to_append(
        &mut self,
        path_identifier: &[String],
    ) -> Result<Option<NodeId>, Action> {
        let selection = path_identifier.last().unwrap();
        let tree = self.get_tree();
        let found_node = tree.nodes().find(|node| node.value() == selection);

        match found_node {
            Some(node) if node.has_children() => Ok(None),
            Some(node) => Ok(Some(node.id())),
            None => {
                let message = format!("Not able to find tree item at {}", selection);
                Err(Action::Error(message))
            }
        }
    }
    fn get_tree(&mut self) -> Tree<String> {
        Tree::new("this".to_string())
    }
    fn create_tree_item_path(&mut self, tree_item_path: &mut Vec<String>, selection: Option<&str>) -> Option<&String> {
        tree_item_path.push(selection.unwrap().to_string());

        // find node
        let tree = self.get_tree();
        let parent_node = tree
            .nodes()
            .find(|node| node.value() == selection.unwrap())
            .unwrap()
            .parent();

        match parent_node {
            Some(parent) => self.create_tree_item_path(tree_item_path, Some(parent.value())),
            None => None,
        }
        
    }
}
