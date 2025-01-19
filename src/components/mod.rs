use std::io::Result;

use crate::{action::Action, app::Focus, config::Config, key::Key};
use crossterm::event::MouseEvent;
use ratatui::{layout::Rect, Frame};

pub mod connections;
pub mod footer;
pub mod results_pager;
pub mod viewer;

// pub trait DrawableComponent {
//     fn draw<B: Backend>(&self, f: &mut Frame, area: Rect, focused: bool) -> Result<()>;
// }

// pub trait StatefulDrawableComponent {
//     fn draw(&mut self, frame: &mut Frame, area: Rect, focused: bool) -> Result<()>;
// }

pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: Focus) -> Result<()>;
    // fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
    //     let action = match event {
    //         Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
    //         Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
    //         _ => None,
    //     };

    //     Ok(action)
    // }

    fn handle_key_event(&mut self, key: Key, focus: Focus) -> Result<Option<Action>> {
        let _key = key;
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
    fn register_config(&mut self, config: Config) -> Result<()>;
    fn list_items(&mut self, data: Vec<u8>, path: Vec<String>) -> Result<()> {
        let _data = data;
        let _path = path;
        Ok(())
    }
    // fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<Option<Action>>;
}
