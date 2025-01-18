use crate::key::Key;

#[derive(Debug, Clone, Copy)]
pub struct KeyConfig {
    pub quit: Key,
    pub change_focus: Key,
    pub exit: Key,
    pub key_down: Key,
    pub key_up: Key,
    pub key_left: Key,
    pub key_right: Key,
    pub arrow_down: Key,
    pub arrow_up: Key,
    pub arrow_left: Key,
    pub arrow_right: Key,
    pub activate_connection: Key,
    pub select_last: Key,
    pub select_first: Key,
    pub next_page: Key,
    pub previous_page: Key,
    pub toggle_selected: Key,
    pub unselect: Key, // ESC
    pub page_down: Key,
    pub page_up: Key,
    pub list_item: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            quit: Key::Char('q'),
            exit: Key::Ctrl('c'),
            key_down: Key::Char('j'),
            key_up: Key::Char('k'),
            key_left: Key::Char('h'),
            key_right: Key::Char('l'),
            arrow_down: Key::Down,
            arrow_up: Key::Up,
            arrow_left: Key::Left,
            arrow_right: Key::Right,
            activate_connection: Key::Char('a'),
            select_last: Key::Ctrl('j'),
            select_first: Key::Ctrl('k'),
            next_page: Key::Ctrl('l'),
            previous_page: Key::Ctrl('h'),
            toggle_selected: Key::Char(' '),
            unselect: Key::Esc,
            page_down: Key::Ctrl('f'),
            page_up: Key::Ctrl('b'),
            list_item: Key::Enter,
            change_focus: Key::Tab,
        }
    }
}
