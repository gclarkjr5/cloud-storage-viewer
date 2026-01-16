use std::io::BufRead;
use std::result::Result;

use crossterm::event::{KeyEvent, MouseEventKind};
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
use tracing::info;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::action::Action;
use crate::app::Focus;
use crate::config::Config;
use crate::config::cloud_provider_config::cloud_provider_connection::CloudConnection;
use crate::key::Key;
use crate::util;

use super::filter::{Filter, ViewerFilter};
use super::results_pager::ResultsPager;
use super::{Component, TreeComponent};

#[derive(Debug)]
pub struct Viewer {
    pub config: Config,
    pub state: TreeState<String>,
    pub tree: ETree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub results_pager: ResultsPager,
    pub pagers: Vec<ResultsPager>,
    pub filter: Box<dyn Filter>,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            config: Config::default(),
            state: TreeState::default(),
            tree: ETree::new("".to_string()),
            items: Vec::new(),
            results_pager: ResultsPager::default(),
            pagers: Vec::new(),
            filter: Box::new(ViewerFilter::default()),
        }
    }
}

impl Viewer {
    fn create_nodes(&mut self, config: &Config, node_id: NodeId) -> Result<(), Action> {
            let ac = config.cloud_provider_config.active_cloud_connection.clone().expect("error viewer");
            if let CloudConnection::Gcs(c) = ac {
                 c.data.expect("error getting data").lines().for_each(|result| {
                     let res = result.expect("error getting result");

                     match self.tree.get_mut(node_id) {
                         None => (),
                         Some(mut tree) => {tree.append(res);}
                     }
                });
                Ok(())
                
            } else {
                Ok(())
            }
            // match cloud_provider {
            //     CloudProviderKind::Azure => {
            //         config.cloud_provider_config.azure.iter().for_each(|config| {
            //             let res = config.name.to_string();

            //             match self.tree.get_mut(node_id) {
            //                 None => (),
            //                 Some(mut tree) => {tree.append(res);}
            //             }
            //         });
            //         Ok(())
                
            //     },
            //     CloudProviderKind::Gcs => {
            //         config.cloud_provider_config.gcs.iter().for_each(|config| {
            //             let res = config.name.to_string();

            //             match self.tree.get_mut(node_id) {
            //                 None => (),
            //                 Some(mut tree) => {tree.append(res);}
            //             }
            //         });
            //         Ok(())
            //     }
            //     CloudProviderKind::S3 => Err(Action::Error("S3 not implemented".to_string())),
            // }
        
    }
    pub fn increase_results_page(&mut self) -> Option<()> {
        // only increase page idx if we are on a page less than the number of pages
        if self.results_pager.page_idx + 1 < self.results_pager.num_pages {
            info!("Going up a page.");
            self.results_pager.page_idx += 1;
            Some(())
        } else {
            None
        }
    }

    pub fn decrease_results_page(&mut self) -> Option<()> {
        // only decrease page idx if we are on a page higher than 1
        if self.results_pager.page_idx + 1 > 1 {
            info!("Going down a page.");
            self.results_pager.page_idx -= 1;
            Some(())
        } else {
            None
        }
    }

}


impl Component for Viewer {
    fn name(&self) -> &str {
        "Viewer"
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn register_config(&mut self, config: &Config, focus: Focus) -> Result<(), String> {
        if config.app_selection.is_empty() {
            return Ok(())
        }

        let initial_app_selection = &config.app_selection[0];

        match (focus, initial_app_selection.as_str()) {
            (Focus::Connections, _) => {
                Ok(())
            }
            (Focus::Viewer, "Cloud Providers") => {
                // This is a request to list a connection from the Connections Component
                let results_pager = ResultsPager::default();

                let active_config = match &config.cloud_provider_config.active_cloud_connection {
                    None => "No Active Cloud Connection".to_string(),
                    Some(s) => s.to_string()
                };

                self.tree = ETree::new(active_config);

                let mut items = vec![];
                    // let nodes = tree.nodes();

                let cloud = &config.app_selection[1];
                let conn = &config.app_selection[2];

                let node_str = format!("{}({})", conn, cloud);

                let found_node = self.find_node_to_append(&[node_str]).expect("Error finding node");

                match found_node {
                    None => {
                        info!("No Tree Node Identified");
                    },
                    Some(nid) => {
                        info!("Tree Node Identified. Creating Stateful Tree for {:?}", &config.app_selection);
                        self.create_nodes(config, nid).expect("Error Creating Nodes");

                        self.items =
                            util::make_tree_items(self.tree.nodes(), &mut self.results_pager, Focus::Connections);

                        self.state.open(config.app_selection.to_vec());
                
                    }
                }
                self.tree.nodes()
                    .filter(|node| node.parent().is_none())
                    .for_each(|node| {
                        let val = node.value().to_string();
                        let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                            .expect("error creating nodes under parent");

                        util::add_children(node, &mut ti, &mut results_pager.clone(), focus);
                        items.push(ti);
                    });
            
                // self.tree = tree;
                self.items = items;
                self.results_pager = results_pager;
                self.filter.register_config(config, focus)?;
                Ok(())
            }
            (Focus::Viewer, _) => {
                let mut items = vec![];
                let results_pager = ResultsPager::default();
                let found_node = self.find_node_to_append(&config.app_selection).expect("Error finding node");
                match found_node {
                    None => {
                        info!("No Tree Node Identified");
                    },
                    Some(nid) => {
                        info!("Tree Node Identified. Adding to tree {:?}", &config.app_selection);
                        self.create_nodes(config, nid).expect("Error Creating Nodes");


                        self.items =
                            util::make_tree_items(self.tree.nodes(), &mut self.results_pager, Focus::Connections);

                        self.state.open(config.app_selection.to_vec());
                
                    }
                }
                self.tree.nodes()
                    .filter(|node| node.parent().is_none())
                    .for_each(|node| {
                        let val = node.value().to_string();
                        let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
                            .expect("error creating nodes under parent");

                        util::add_children(node, &mut ti, &mut results_pager.clone(), focus);
                        items.push(ti);
                    });
                self.items = items;
                self.results_pager = results_pager;
                self.filter.register_config(config, focus)?;
                Ok(())
            }
            (_, _) => {
                Ok(())
            }
        }

        // } else {
            // The request was made from the Viewer
            // let found_node = self.find_node_to_append(&config.app_selection).expect("Error finding node");
            // info!("Found this node from the viewer: {:?}", found_node);
            // match found_node {
            //     None => {
            //         info!("No Tree Node Identified");
            //     },
            //     Some(nid) => {
            //         info!("Tree Node Identified. Creating Stateful Tree for {:?}", &config.app_selection);
            //         self.create_nodes(config, nid).expect("Error Creating Nodes");

            //         self.items =
            //             util::make_tree_items(self.tree.nodes(), &mut self.results_pager, Focus::Connections);

            //         self.state.open(config.app_selection.to_vec());
                
            //     }
            // }
            // self.tree.nodes()
            //     .filter(|node| node.parent().is_none())
            //     .for_each(|node| {
            //         let val = node.value().to_string();
            //         let mut ti = TreeItem::new(val.clone(), val.clone(), vec![])
            //             .expect("error creating nodes under parent");

            //         util::add_children(node, &mut ti, &mut results_pager.clone(), focus);
            //         items.push(ti);
            //     });
        // }

    }

    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::Viewer => {
                if [self.config.key_config.quit, self.config.key_config.exit]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.change_focus {
                    Ok(Action::ChangeFocus(Focus::Connections))
                } else if [
                    self.config.key_config.key_up,
                    self.config.key_config.arrow_up,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_up();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_down,
                    self.config.key_config.arrow_down,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_down();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_left,
                    self.config.key_config.arrow_left,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_left();
                    Ok(Action::Nothing)
                } else if [
                    self.config.key_config.key_right,
                    self.config.key_config.arrow_right,
                ]
                .iter()
                .any(|kc| kc == &key)
                {
                    self.state.key_right();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.select_last {
                    self.state.select_last();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.select_first {
                    self.state.select_first();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.toggle_selected {
                    self.state.toggle_selected();
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.list_item {
                    let selected = self.state.selected().to_vec();
                    Ok(Action::ViewerList(selected))
                    // let actual_request_path = selected.last().unwrap();
                    // let data = util::cli_command("gsutil", &vec!["ls", actual_request_path])?;
                    // self.list_item(data, selected, focus)
                    //     .expect("error calling list items");
                    // Ok(Action::Nothing)
                } else if key == self.config.key_config.next_page {
                    self.increase_results_page();
                    self.items =
                        util::make_tree_items(self.tree.nodes(), &mut self.results_pager, focus);
                    self.state.select(self.results_pager.paged_item.clone());
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.previous_page {
                    self.decrease_results_page();
                    self.items =
                        util::make_tree_items(self.tree.nodes(), &mut self.results_pager, focus);
                    self.state.select(self.results_pager.paged_item.clone());
                    Ok(Action::Nothing)
                } else if key == self.config.key_config.filter {
                    // activate filter
                    self.filter.switch_active_status();
                    Ok(Action::ChangeFocus(Focus::ViewerFilter))
                    // Ok(Some(Action::Nothing))
                } else {
                    Ok(Action::Nothing)
                }
            }
            Focus::ViewerFilter => {
                let action = self.filter.handle_key_event(key_event, focus)?;
                match action {
                    Action::Filter(txt) => {
                        let tree_items = self
                            .tree
                            .nodes()
                            .filter(|n| n.value().contains('/'))
                            .map(|n| n.value().to_string())
                            .collect();

                        self.filter.engage_filter(txt, tree_items)
                    }
                    _ => Ok(action),
                }
            }
            Focus::ViewerFilterResults => self
                .filter
                .filter_results_handle_key_event(key_event, focus),
            _ => Ok(Action::Skip),
        }
    }

    fn handle_mouse_event(
        &mut self,
        mouse_event: crossterm::event::MouseEvent,
        focus: Focus,
    ) -> Result<Action, Action> {
        match focus {
            Focus::Viewer => match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    self.state.scroll_down(1);
                    Ok(Action::Nothing)
                }
                MouseEventKind::ScrollUp => {
                    self.state.scroll_up(1);
                    Ok(Action::Nothing)
                }
                MouseEventKind::Down(_button) => {
                    self.state
                        .click_at(Position::new(mouse_event.column, mouse_event.row));
                    Ok(Action::Nothing)
                }
                _ => Ok(Action::Nothing),
            },
            _ => Ok(Action::Nothing),
        }
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        focus: crate::app::Focus,
        _config: &Config,
    ) -> Result<(), String> {
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

        // if self.results_pager.num_pages > 1 {
        let paging_info = format!(
            "currently paging: {}
                    page: {} of {}
                    showing: {} of {}",
            if let Some(item) = self.results_pager.paged_item.last() {
                item
            } else {
                "Nothing being paged"
            },
            // self.results_pager.paged_item.last().unwrap(),
            self.results_pager.page_idx + 1,
            self.results_pager.num_pages,
            // self.results_pager.results_per_page,
            match self.results_pager.page_idx {
                0 => format!("{}-{}", 0, ((self.results_pager.page_idx + 1) * self.results_pager.results_per_page)),
                _ => format!("{}-{}", ((self.results_pager.page_idx + 1) * self.results_pager.results_per_page) + 1, ((self.results_pager.page_idx + 2) * self.results_pager.results_per_page)),
            },
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
            paging_area,
        );
        // }
        self.filter.draw(frame, viewer, focus)?;
        Ok(())
    }
}

impl TreeComponent for Viewer {
    fn get_tree(&mut self) -> ETree<String> {
        self.tree.clone()
    }

    fn list_item(&mut self, data: Vec<u8>, path_identifier: Vec<String>, focus: Focus) -> Result<(), Action> {
        // find node, verify, unwrap, and set pager
        let found_node = self.find_node_to_append(&path_identifier)?;

        match found_node {
            None => Ok(()),
            Some(node_id) => {
                let selection = path_identifier.last().unwrap();
                let is_directory =
                    selection.chars().last().expect("error getting last char") == '/';

                match is_directory {
                    true => {
                        add_tree_items(data.clone(), &mut self.tree, node_id);
                    }
                    false => {
                        let root = self.tree.root().value();
                        if root == selection {
                            
                            add_tree_items(data.clone(), &mut self.tree, node_id);
                        } else {
                            {}
                        }
                        // match root == selection {
                        //     true => {
                        //     }
                        //     false => {}
                        // }
                    }
                }

                // remake tree widget
                self.results_pager.init(&data, path_identifier.clone());
                self.pagers.push(self.results_pager.clone());
                self.items =
                    util::make_tree_items(self.tree.nodes(), &mut self.results_pager, focus);

                self.state.open(path_identifier.clone());
                self.state.select(path_identifier);

                Ok(())
            }
        }
    }

    fn select_item(&mut self, selection: &str, focus: Focus) -> Result<(), String> {
        if matches!(focus, Focus::Viewer) {
            let mut tree_item_path: Vec<String> = vec![];

            self.create_tree_item_path(&mut tree_item_path, Some(selection));

            tree_item_path.reverse();

            // is the parent of the selection == to the results pager
            let selection_parent = self
                .tree
                .nodes()
                .find(|n| n.value() == selection)
                .unwrap()
                .parent()
                .unwrap();
            let parent_is_current_pager =
                selection_parent.value() == self.results_pager.paged_item.last().unwrap();

            match parent_is_current_pager {
                true => {
                    // if so, more than 1 page?
                    if self.results_pager.num_pages > 1 {
                        // find which page
                        let mut new_page_idx = 0;
                        let children: Vec<NodeRef<String>> = selection_parent.children().collect();
                        children
                            .chunks(self.results_pager.results_per_page)
                            .enumerate()
                            .for_each(|(chunk_idx, chunk)| {
                                if chunk.iter().any(|n| n.value() == selection) {
                                    new_page_idx = chunk_idx;
                                }
                            });

                        // set the page, re-list-items
                        self.results_pager.set_page_idx(new_page_idx);
                        self.items = util::make_tree_items(
                            self.tree.nodes(),
                            &mut self.results_pager,
                            focus,
                        );
                    }
                }
                false => {
                    // while not currently paging this parent, does it have multiple pages?
                    if selection_parent.children().count() > self.results_pager.results_per_page {
                        // find which page
                        let mut new_page_idx = 0;
                        let children: Vec<NodeRef<String>> = selection_parent.children().collect();
                        children
                            .chunks(self.results_pager.results_per_page)
                            .enumerate()
                            .for_each(|(chunk_idx, chunk)| {
                                if chunk.iter().any(|n| n.value() == selection) {
                                    new_page_idx = chunk_idx;
                                }
                            });

                        let other_pager = self
                            .pagers
                            .iter()
                            .find(|p| p.paged_item.last().unwrap() == selection_parent.value())
                            .unwrap()
                            .clone();

                        self.results_pager = other_pager;

                        // set the page, re-list-items
                        self.results_pager.set_page_idx(new_page_idx);
                        self.items = util::make_tree_items(
                            self.tree.nodes(),
                            &mut self.results_pager,
                            focus,
                        );
                    }
                }
            }
            // if not, check if its parent has multiple pages currently listed
            // set results pager to that parent
            // get idx of selection within parent

            self.filter.switch_active_status();
            self.state.select(tree_item_path);
        }
        Ok(())
    }
    
}

pub fn add_tree_items(data: Vec<u8>, tree: &mut ETree<String>, node_id: NodeId) {
    data.lines().for_each(|listing| {
        let res = listing.expect("error getting listing");
        tree.get_mut(node_id)
            .expect("error getting mutable node")
            .append(res);
    });
}
