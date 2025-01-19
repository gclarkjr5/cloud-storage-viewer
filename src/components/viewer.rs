use std::{io::BufRead, io::Result, process::Command};

use crossterm::event::MouseEventKind;
use ego_tree::{NodeId, NodeRef, Tree as ETree};
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::{
    layout::Rect,
    widgets::{Block, Scrollbar, ScrollbarOrientation},
    Frame,
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::action::Action;
use crate::app::Focus;
use crate::config::Config;
use crate::key::Key;

use super::results_pager::ResultsPager;
use super::Component;

pub struct Viewer {
    pub config: Config,
    pub state: TreeState<String>,
    pub tree: ETree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub results_pager: ResultsPager,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            config: Config::default(),
            state: TreeState::default(),
            tree: ETree::new("".to_string()),
            items: Vec::new(),
            results_pager: ResultsPager::default(),
        }
    }
}

impl Viewer {
    pub fn _update(active_connection: &str) -> Self {
        let tree = ETree::new(active_connection.to_string());
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
            config: Config::default(),
            state: TreeState::default(),
            tree: ETree::new(active_connection.to_string()),
            items: Vec::new(),
            results_pager: ResultsPager::default(),
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

    pub fn find_node_to_append(&mut self, path: Vec<String>) -> Option<(String, NodeId)> {
        // use the selction to find the node in the tree
        let selection = path.last().unwrap();

        let found_node = self.tree.nodes().find(|node| node.value() == selection);

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
        if node.has_children() {
            return None;
        }

        // return the selction and the node id
        Some((selection.clone(), node.id()))
    }
}

pub fn cli_command(program: &str, args: Vec<&str>) -> Vec<u8> {
    Command::new(program)
        .args(args)
        .output()
        .expect("error processing command")
        .stdout
}

impl Component for Viewer {
    fn init(&mut self) -> Result<()> {
        // let active_connection = self.config.name.clone();
        // let active_conn = self
        //     .config
        //     .cloud_config
        //     .available_clouds
        //     .iter()
        //     .find(|a| matches!(a, Cloud::Gcs(_)))
        //     .unwrap();

        // let mut g = GcsConn::default();
        // if let Cloud::Gcs(config) = active_conn {
        //     g = config.clone().active_conn;
        // }
        // let tree = ETree::new(g.name.to_string());
        // let mut items = vec![];
        // let results_pager = ResultsPager::default();

        // let nodes = tree.nodes();
        // nodes
        //     .filter(|node| node.parent().is_none())
        //     .for_each(|node| {
        //         let val = node.value().to_string();
        //         let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
        //             .expect("error creating nodes under parent");

        //         add_children(node, &mut ti, &mut results_pager.clone());
        //         items.push(ti);
        //     });

        // self.tree = tree;
        // self.items = items;
        // self.results_pager = results_pager;

        Ok(())
    }

    fn list_items(&mut self, data: Vec<u8>, path: Vec<String>) -> Result<()> {
        // find node, verify, unwrap, and set pager
        let found_node = self.find_node_to_append(path.clone());

        if found_node.is_none() {
            return Ok(());
        }

        let (selection, node_to_append_to) = found_node.unwrap();

        let is_directory = selection.chars().last().expect("error getting last char") == '/';

        self.results_pager.init(&data, path.clone());

        match is_directory {
            true => {
                data.lines().for_each(|listing| {
                    let res = listing.expect("error getting listing from stdout");
                    self.tree
                        .get_mut(node_to_append_to)
                        .expect("error getting mutable node")
                        .append(res);
                });
            }
            false => {
                let root = self.tree.root().value();
                match root == &selection {
                    true => {
                        data.lines().for_each(|listing| {
                            let res = listing.expect("error getting listing from stdout");
                            self.tree
                                .get_mut(node_to_append_to)
                                .expect("error getting mutable node")
                                .append(res);
                        });
                    }
                    false => {}
                }
            }
        }

        // remake tree widget
        self.items = self.make_items();

        self.state.open(path.clone());
        self.state.select(path);

        Ok(())
    }

    fn register_config(&mut self, config: Config) -> Result<()> {
        self.config = config;

        let active_config = format!("{}", self.config.cloud_config);

        let tree = ETree::new(active_config);
        let mut items = vec![];
        let results_pager = ResultsPager::default();

        let nodes = tree.nodes();
        nodes
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                    .expect("error creating nodes under parent");

                add_children(node, &mut ti, &mut results_pager.clone());
                items.push(ti);
            });

        self.tree = tree;
        self.items = items;
        self.results_pager = results_pager;
        Ok(())
    }

    fn handle_key_event(&mut self, key: Key, focus: Focus) -> Result<Option<Action>> {
        match focus {
            Focus::Viewer => {
                if [self.config.key_config.quit, self.config.key_config.exit]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    Ok(Some(Action::Quit))
                } else if key == self.config.key_config.change_focus {
                    Ok(Some(Action::ChangeFocus))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_up();
                    Ok(None)
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_down();
                    Ok(None)
                } else if [
                    self.config.key_config.key_left,
                    self.config.key_config.arrow_left,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_left();
                    Ok(None)
                } else if [
                    self.config.key_config.key_right,
                    self.config.key_config.arrow_right,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_right();
                    Ok(None)
                } else if key == self.config.key_config.select_last {
                    self.state.select_last();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.select_first {
                    self.state.select_first();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.toggle_selected {
                    self.state.toggle_selected();
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.list_item {
                    let selected = self.state.selected().to_vec();
                    let data = cli_command("gsutil", vec!["ls", selected.last().unwrap()]);
                    self.list_items(data, selected)?;
                    Ok(None)
                } else if key == self.config.key_config.next_page {
                    self.increase_results_page();
                    self.items = self.make_items();
                    self.state.select(self.results_pager.paged_item.clone());
                    Ok(Some(Action::Nothing))
                } else if key == self.config.key_config.previous_page {
                    self.decrease_results_page();
                    self.items = self.make_items();
                    self.state.select(self.results_pager.paged_item.clone());
                    Ok(Some(Action::Nothing))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn handle_mouse_event(
        &mut self,
        mouse_event: crossterm::event::MouseEvent,
        focus: Focus,
    ) -> Result<Option<Action>> {
        match focus {
            Focus::Viewer => match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    self.state.scroll_down(1);
                    Ok(Some(Action::Nothing))
                }
                MouseEventKind::ScrollUp => {
                    self.state.scroll_up(1);
                    Ok(Some(Action::Nothing))
                }
                MouseEventKind::Down(_button) => {
                    self.state
                        .click_at(Position::new(mouse_event.column, mouse_event.row));
                    Ok(Some(Action::Nothing))
                }
                _ => Ok(Some(Action::Nothing)),
            },
            _ => Ok(Some(Action::Nothing)),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: crate::app::Focus) -> Result<()> {
        let focused = matches!(focus, Focus::Viewer);
        let [content, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let [_, viewer] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        let widget = Tree::new(&self.items)
            .expect("all item identifieers are unique")
            .block(
                Block::bordered()
                    .title("Cloud Viewer")
                    .border_style(if focused {
                        Style::new().blue()
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(if focused {
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
            ));

        frame.render_widget(Clear, viewer);
        frame.render_stateful_widget(widget, viewer, &mut self.state);

        if self.results_pager.num_pages > 1 {
            let paging_info = format!(
                "currently paging: {}
                    page: {} of {}
                    showing: {} of {}",
                self.results_pager.paged_item.last().unwrap(),
                self.results_pager.page_idx + 1,
                self.results_pager.num_pages,
                self.results_pager.results_per_page,
                self.results_pager.total_results,
            );
            #[allow(clippy::cast_possible_truncation)]
            let paging_area = Rect {
                y: viewer.height - 2,
                height: 10,
                x: frame.area().width.saturating_sub(paging_info.len() as u16),
                width: paging_info.len() as u16,
            };
            frame.render_widget(
                Span::styled(paging_info, Style::new().fg(Color::Black).bg(Color::Gray)),
                // Paragraph::new(text),
                paging_area,
            );
        }
        Ok(())
    }
}

fn add_children(
    node: NodeRef<String>,
    tree_item: &mut TreeItem<String>,
    results_pager: &mut ResultsPager,
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
