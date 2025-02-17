use std::result::Result;

use crate::{action::Action, app::Focus, config::Config};
use crossterm::event::{KeyEvent, MouseEvent};
use ego_tree::{NodeId, Tree};
use ratatui::{layout::Rect, Frame};

pub mod connection_filter;
pub mod connection_filter_results;
pub mod connections;
pub mod error;
pub mod footer;
pub mod results_pager;
pub mod viewer;
pub mod viewer_filter;
pub mod viewer_filter_results;

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
    fn list_item(&mut self, data: Vec<u8>, path: Vec<String>, focus: Focus) -> Result<(), Action> {
        let _data = data;
        let _path = path;
        let _focus = focus;
        Ok(())
    }
    // fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<Option<Action>>;
    fn select_item(&mut self, item: &str, focus: Focus) -> Result<(), String> {
        let _item = item;
        let _focus = focus;
        Ok(())
    }

    fn report_error(&mut self, message: String) -> Result<(), String> {
        let _message = message;
        // let _foucs = focus;
        Ok(())
    }

    fn find_node_to_append(
        &mut self,
        path_identifier: &[String],
    ) -> Result<Option<NodeId>, Action>
        // where
        //     Self: TreeComponent
    {
        // let _path_identifier = path_identifier;
        // Ok(None)
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

}

// pub trait TreeComponent {
//     fn get_tree(&mut self) -> Tree<String> {
//         Tree::new("this".to_string())
//     }
    
// }
