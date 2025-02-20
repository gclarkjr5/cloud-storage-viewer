use super::filter::{ConnectionFilter, Filter};
use super::results_pager::ResultsPager;
use super::{Component, TreeComponent};
use crossterm::event::{KeyEvent, MouseEventKind};
use ego_tree::Tree as ETree;
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config as NucleoConfig, Nucleo};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::block::Block;
use ratatui::widgets::{Clear, Scrollbar, ScrollbarOrientation};

use ratatui::Frame;
use std::result::Result;
use std::sync::Arc;
use std::vec;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::action::Action;
use crate::app::Focus;
use crate::config::Config;
use crate::key::Key;
use crate::util;

// #[derive(Debug)]
pub struct Connections {
    pub state: TreeState<String>,
    pub tree: ETree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub config: Config,
    pub results_pager: ResultsPager,
    pub filter: Box<dyn Filter>,
    // pub filter: ConnectionFilter,
}

impl Default for Connections {
    fn default() -> Self {
        Self::new()
    }
}


impl Connections {
    fn create_tree_item_path(
        &self,
        tree_item_path: &mut Vec<String>,
        selection: Option<&str>,
    ) -> Option<&String> {
        // add path
        tree_item_path.push(selection.unwrap().to_string());

        // find node
        let parent_node = self
            .tree
            .nodes()
            .find(|node| node.value() == selection.unwrap())
            .unwrap()
            .parent();

        match parent_node {
            Some(parent) => self.create_tree_item_path(tree_item_path, Some(parent.value())),
            None => None,
        }
    }
    pub fn new() -> Self {
        Self {
            state: TreeState::default(),
            tree: ETree::new(String::new()),
            items: Vec::new(),
            config: Config::default(),
            results_pager: ResultsPager::default(),
            filter: Box::new(ConnectionFilter::default()),
        }
    }

    pub fn activate_connection(&mut self, selection: Vec<String>) -> Result<Action, Action> {
        // verify that the selected connection is for an implemented cloud
        // technically should never happen, mainly to see how far development
        // of more cloud implementations has gotten
        let _cloud_provider = self.config
            .cloud_config
            .verify_implemented_cloud_provider(selection.clone())?;

        // first is the root, second is the cloud provider
        if selection.len() < 3 {
            // then we only have root + provider
            Err(Action::Error("Cannot activate a Cloud Provider, please select one of its accounts".to_string()))
        } else {
            self.config.cloud_config.activate_config(selection)
        }
    }

    pub fn list_cloud_provider(
        &mut self,
        selection: Vec<String>,
        focus: Focus,
    ) -> Result<Action, Action> {
        let mut cloud_provider = self
            .config
            .cloud_config
            .verify_implemented_cloud_provider(selection.clone())?;

        // find the node to append to
        let found_node = self.find_node_to_append(&selection)?;
        // let found_node = util::find_node_to_append(&mut self.tree, &selection)?;

        // if empty dont do anything
        match found_node {
            None => Ok(Action::Nothing),
            Some(nid) => {
                cloud_provider.update()?;
                cloud_provider.create_nodes(&mut self.tree, nid)?;
                self.items =
                    util::make_tree_items(self.tree.nodes(), &mut self.results_pager, focus);
                self.state.open(selection.clone());
                Ok(Action::Nothing)
            }
        }
    }

    pub fn list_configuration(&mut self, path_identifier: Vec<String>) -> Result<Action, Action> {
        // set active cloud
        self.config.cloud_config.activate_config(path_identifier)?;

        let output = util::cli_command("gsutil", &vec!["ls"])?;

        Ok(Action::ListConfiguration(
            self.config.cloud_config.clone(),
            // vec![format!("{}", self.config.cloud_config)],
            output,
        ))
    }
}

impl Component for Connections {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self) -> Result<(), String> {
        let mut tree = ETree::new("Connections".to_string());

        self.config
            .cloud_config
            .cloud_providers
            .iter()
            .for_each(|cloud| {
                tree.root_mut().append(cloud.to_string());
            });

        let mut items = vec![];

        tree.nodes()
            .filter(|node| node.parent().is_none())
            .for_each(|node| {
                let val = node.value().to_string();
                if let Ok(mut ti) = TreeItem::new(val.clone(), val.clone(), vec![]) {
                    let mut results_pager = self.results_pager.clone();

                    util::add_children(node, &mut ti, &mut results_pager, Focus::Connections);
                    items.push(ti);
                }
            });

        self.tree = tree;
        self.items = items;

        Ok(())
    }


    fn handle_mouse_event(
        &mut self,
        mouse_event: crossterm::event::MouseEvent,
        focus: Focus,
    ) -> Result<Action, Action> {
        match focus {
            Focus::Connections => match mouse_event.kind {
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
    fn handle_key_event(&mut self, key_event: KeyEvent, focus: Focus) -> Result<Action, Action> {
        let key: Key = key_event.into();
        match focus {
            Focus::Connections => {
                if [self.config.key_config.quit, self.config.key_config.exit]
                    .iter()
                    .any(|kc| kc == &key)
                {
                    Ok(Action::Quit)
                } else if key == self.config.key_config.change_focus {
                    Ok(Action::ChangeFocus(Focus::Viewer))
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
                } else if key == self.config.key_config.activate_connection {
                    let selected = self.state.selected().to_vec();
                    self.activate_connection(selected)
                } else if key == self.config.key_config.list_item {
                    let selected = self.state.selected().to_vec();
                    // self.list_item(selection)
                    if selected.len() == 2 {
                        // listing a cloud provider
                        self.list_cloud_provider(selected, Focus::Connections)
                    } else if selected.len() == 3 {
                        // listing a config
                        self.list_configuration(selected.clone())
                    } else {
                        Ok(Action::Nothing)
                    }
                } else if key == self.config.key_config.filter {
                    // activate filter
                    // self.filter.active = !self.filter.active;
                    self.filter.active();
                    Ok(Action::ChangeFocus(Focus::ConnectionsFilter))
                } else {
                    Ok(Action::Nothing)
                }
            }
            Focus::ConnectionsFilter => {
                let action = self.filter.handle_key_event(key_event, focus)?;
                match action {
                    Action::Filter(txt) => {
                        let tree_items = self
                            .tree
                            .nodes()
                            .filter(|n| !n.has_children())
                            .map(|n| n.value().to_string())
                            .collect();

                        self.filter.set_filter_result_items(tree_items);

                        let number_of_columns = 1;

                        let mut nucleo = Nucleo::new(
                            NucleoConfig::DEFAULT,
                            Arc::new(|| {}),
                            None,
                            number_of_columns,
                        );

                        // Send the strings to search through to the matcher
                        let injector = nucleo.injector();

                        let filtered_result_items = self.filter.get_filter_result_items();

                        // println!("Filtered result items are {:?}", filtered_result_items);
                        for (id, string) in filtered_result_items
                            .iter()
                            .enumerate()
                        {
                            // Only the strings assigned to row in the closure below are matched on,
                            // so it's possible to pass an identifier in.
                            let item = (id, string.to_owned());

                            injector.push(item, |(_id, string), row| {
                                // The size of this array is determined by number_of_columns
                                let str_clone = string.clone();
                                row[0] = str_clone.into()
                            });
                        }

                        if let Some(search_term) = txt.last() {
                        // let search_term = txt.last().unwrap();
                            nucleo.pattern.reparse(
                                0,
                                search_term,
                                CaseMatching::Ignore,
                                Normalization::Smart,
                                false,
                            );
                        }

                        // The search is initialised here...

                        // ...but actually begins here
                        let _status = nucleo.tick(500);
                        // if status.changed {
                        //     println!("There are new results.")
                        // }
                        // if !status.running {
                        //     println!("The search has finished.")
                        // }

                        // Snapshot contains the current set of results
                        let snapshot = nucleo.snapshot();

                        // Matching items are returned, ranked by highest score first.
                        // These are just the items as pushed to the injector earlier.
                        let matches: Vec<_> = snapshot.matched_items(..).collect();

                        let mut data_list: Vec<String> = vec![];
                        for item in matches {
                            let (_, data) = item.data;

                            data_list.push(data.to_string());
                        }
                        // self.filter.filtered_results.filtered_items = data_list.clone();

                        // set filtered items to the data list
                        self.filter.set_filter_result_filtered_items(data_list.clone());

                        // gather the filtered items
                        let filtered_items = self.filter.get_filter_result_filtered_items().clone();

                        // add filtered items to the results.items()
                        let filter_result_items = self.filter.get_filter_result_results().clone().items(filtered_items.clone());

                        // set filtered results to the items above
                        self.filter.set_filter_result_results(filter_result_items);

                        Ok(Action::Nothing)
                    }
                    _ => Ok(action),
                }
            }
            Focus::ConnectionFilterResults => self
                .filter
                .filter_results_handle_key_event(key_event, focus),
            _ => Ok(Action::Nothing),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, focus: Focus) -> Result<(), String> {
        let focused = matches!(focus, Focus::Connections);
        let [content, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(area);

        let [connections, _viewer] =
            Layout::horizontal([Constraint::Percentage(15), Constraint::Min(1)]).areas(content);

        match Tree::new(&self.items) {
            Ok(tree) => {
                let widget =
                    tree.block(Block::bordered().title("Cloud Connections").border_style(
                        if focused {
                            Style::new().blue()
                        } else {
                            Style::default()
                        },
                    ))
                    .highlight_style(if focused {
                        Style::new()
                            .fg(Color::Black)
                            .bg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    })
                    .experimental_scrollbar(Some(
                        Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                            .begin_symbol(None)
                            .track_symbol(None)
                            .end_symbol(None),
                    ));

                frame.render_widget(Clear, connections);
                frame.render_stateful_widget(widget, connections, &mut self.state);
                self.filter.draw(frame, connections, focus)?;
                Ok(())
            }
            Err(_) => Err("all item identifiers need to be unique in connection tree".to_string()),
        }
    }

    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String> {
        self.config = config;
        self.filter.register_config(self.config.clone(), focus)
    }
}

impl TreeComponent for Connections {
    fn get_tree(&mut self) -> ETree<String> {
        self.tree.clone()
    }

    fn select_item(&mut self, selection: &str, focus: Focus) -> Result<(), String> {
        if matches!(focus, Focus::Connections) {
            let mut tree_item_path: Vec<String> = vec![];

            self.create_tree_item_path(&mut tree_item_path, Some(selection));

            tree_item_path.reverse();
            // let mut selection = vec!["Connections".to_string()];

            // self.filter.active = !self.filter.active;
            self.filter.switch_active_status();
            // let path = item.split('/').nth(0).unwrap().to_string();
            // selection.push(path);
            // selection.push(item.to_string());

            self.state.select(tree_item_path);
        }
        Ok(())
    }
}
