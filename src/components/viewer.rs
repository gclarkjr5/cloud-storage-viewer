use std::result::Result;
use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEventKind};
use ego_tree::{NodeRef, Tree as ETree};
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config as NucleoConfig, Nucleo};
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
use crate::util;

use super::results_pager::ResultsPager;
use super::viewer_filter::ViewerFilter;
use super::Component;

pub struct Viewer {
    pub config: Config,
    pub state: TreeState<String>,
    pub tree: ETree<String>,
    pub items: Vec<TreeItem<'static, String>>,
    pub results_pager: ResultsPager,
    pub pagers: Vec<ResultsPager>,
    pub filter: ViewerFilter,
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
            filter: ViewerFilter::default(),
        }
    }
}

impl Viewer {
    pub fn increase_results_page(&mut self) -> Option<()> {
        // only increase page idx if we are on a page less than the number of pages
        if self.results_pager.page_idx + 1 < self.results_pager.num_pages {
            self.results_pager.page_idx += 1;
            Some(())
        } else {
            None
        }
    }

    pub fn decrease_results_page(&mut self) -> Option<()> {
        // only decrease page idx if we are on a page higher than 1
        if self.results_pager.page_idx + 1 > 1 {
            self.results_pager.page_idx -= 1;
            Some(())
        } else {
            None
        }
    }

    // pub fn find_node_to_append(&mut self, path: Vec<String>) -> Option<(String, NodeId)> {
    //     // use the selction to find the node in the tree
    //     let selection = path.last().unwrap();

    //     let found_node = self.tree.nodes().find(|node| node.value() == selection);

    //     found_node.as_ref()?;
    //     let node = found_node.expect("error unwrapping found node");

    //     if node.has_children() {
    //         return None;
    //     }

    //     // return the selction and the node id
    //     Some((selection.clone(), node.id()))
    // }
}

// pub fn cli_command(program: &str, args: &Vec<&str>) -> Result<Vec<u8>, Action> {
//     util::cli_command(program, args)
//     match Command::new(program).args(args).output() {
//         Ok(output) => Ok(output.stdout),
//         Err(_) => {
//             let message = [program.to_string(), args.join(" ")].join(" ");
//             Err(Action::Error(message))
//         }
//     }
//     // .expect("error processing command")
//     // .stdout
// }

impl Component for Viewer {
    fn list_item(&mut self, data: Vec<u8>, path: Vec<String>, focus: Focus) -> Result<(), Action> {
        // find node, verify, unwrap, and set pager
        let found_node = util::find_node_to_append(&mut self.tree, &path)?;

        // if found_node.is_none() {
        //     return Ok(());
        // }

        // let (selection, node_to_append_to) = found_node.unwrap();

        match found_node {
            None => Ok(()),
            Some(node_id) => {
                let selection = path.last().unwrap();
                let is_directory =
                    selection.chars().last().expect("error getting last char") == '/';

                match is_directory {
                    true => {
                        util::add_tree_items(data.clone(), &mut self.tree, node_id);
                    }
                    false => {
                        let root = self.tree.root().value();
                        match root == selection {
                            true => {
                                util::add_tree_items(data.clone(), &mut self.tree, node_id);
                            }
                            false => {}
                        }
                    }
                }

                // remake tree widget
                self.results_pager.init(&data, path.clone());
                self.pagers.push(self.results_pager.clone());
                self.items =
                    util::make_tree_items(self.tree.nodes(), &mut self.results_pager, focus);

                self.state.open(path.clone());
                self.state.select(path);

                Ok(())
            }
        }
    }

    fn register_config(&mut self, config: Config, focus: Focus) -> Result<(), String> {
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

                util::add_children(node, &mut ti, &mut results_pager.clone(), focus);
                items.push(ti);
            });

        self.tree = tree;
        self.items = items;
        self.results_pager = results_pager;
        self.filter.register_config(self.config.clone(), focus)
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
                    let actual_request_path = selected.last().unwrap();
                    let data = util::cli_command("gsutil", &vec!["ls", actual_request_path])?;
                    self.list_item(data, selected, focus)
                        .expect("error calling list items");
                    Ok(Action::Nothing)
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
                    self.filter.active = !self.filter.active;
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
                        let search_term = txt.last().unwrap();
                        self.filter.filtered_results.items = self
                            .tree
                            .nodes()
                            .filter(|n| n.value().contains('/'))
                            .map(|n| n.value().to_string())
                            .collect();

                        let number_of_columns = 1;

                        let mut nucleo = Nucleo::new(
                            NucleoConfig::DEFAULT,
                            Arc::new(|| {}),
                            None,
                            number_of_columns,
                        );

                        // Send the strings to search through to the matcher
                        let injector = nucleo.injector();

                        for (id, string) in self
                            .filter
                            .filtered_results
                            .items
                            .clone()
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

                        // The search is initialised here...

                        nucleo.pattern.reparse(
                            0,
                            search_term,
                            CaseMatching::Ignore,
                            Normalization::Smart,
                            false,
                        );

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
                        self.filter.filtered_results.filtered_items = data_list.clone();
                        self.filter.filtered_results.results = self
                            .filter
                            .filtered_results
                            .results
                            .clone()
                            .items(self.filter.filtered_results.filtered_items.clone());
                        Ok(Action::Nothing)
                    }
                    _ => Ok(action),
                }
            }
            Focus::ViewerFilterResults => self
                .filter
                .filtered_results
                .handle_key_event(key_event, focus),
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
            paging_area,
        );
        // }
        self.filter.draw(frame, viewer, focus)?;
        Ok(())
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

            self.filter.active = !self.filter.active;
            self.state.select(tree_item_path);
        }
        Ok(())
    }
}

impl Viewer {
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
}
