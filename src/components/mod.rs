use std::io::Result;

use crate::{action::Action, app::Focus, config::Config};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};

pub mod connection_filter;
pub mod connection_filter_results;
pub mod connections;
pub mod footer;
pub mod results_pager;
pub mod viewer;
pub mod viewer_filter;
pub mod viewer_filter_results;

pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: Focus) -> Result<()>;

    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Option<Action>> {
        let _key_event = key_event;
        let _foucs = focus;
        Ok(None)
    }
    fn handle_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        focus: Focus,
    ) -> Result<Option<Action>> {
        let _mouse_event = mouse_event;
        let _foucs = focus;
        Ok(None)
    }
    fn register_config(&mut self, config: Config, focus: Focus) -> Result<()>;
    fn list_items(&mut self, data: Vec<u8>, path: Vec<String>, focus: Focus) -> Result<()> {
        let _data = data;
        let _path = path;
        let _focus = focus;
        Ok(())
    }
    // fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<Option<Action>>;
    fn select_item(&mut self, item: &str, focus: Focus) -> Result<()> {
        let _item = item;
        let _focus = focus;
        Ok(())
    }
}
