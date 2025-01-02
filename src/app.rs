use std::fmt;

use super::components::connections::Connections;
use super::components::viewer::Viewer;

pub enum Cloud {
    Azure,
    Gcs,
    S3,
}

impl fmt::Display for Cloud {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cloud::Gcs => write!(f, "Google Cloud Storage"),
            Cloud::Azure => write!(f, "Azure Data Lake Storage"),
            Cloud::S3 => write!(f, "AWS S3"),
        }
    }
}

#[derive(Debug)]
pub enum Focus {
    Connections,
    Viewer,
}

#[must_use]
pub struct App {
    pub focus: Focus,
    // pub cloud: Cloud,
    pub connections: Connections,
    pub viewer: Viewer,
}

impl App {
    pub fn new() -> Self {
        Self {
            focus: Focus::Connections,
            // cloud: Cloud::Gcs,
            viewer: Viewer::new("CloudFS"),
            connections: Connections::new(),
        }
    }

    pub fn toggle_screen(&mut self) {
        match self.focus {
            Focus::Connections => self.focus = Focus::Viewer,
            Focus::Viewer => self.focus = Focus::Connections,
        }
    }

    // pub fn empty_viewer(&mut self) {
    //     self.viewer = Viewer::new();
    // }

    pub fn list_items(&mut self, path_identifier: Option<Vec<String>>, action: &str) -> bool {
        match path_identifier {
            None => {}

            Some(path) => match self.focus {
                Focus::Connections => {
                    // list items from connections
                    let items = self.connections.list_items(path.clone());

                    match items {
                        None => false, // current account did not match the requested one
                        Some(_) => {
                            // we hit Enter on a connection that lists buckets
                            self.viewer =
                                Viewer::new(self.connections.active.clone().unwrap().as_str());
                            let viewer_root = self.connections.active.clone();
                            // init pager
                            let selection = path
                                .iter()
                                .last()
                                .expect("error getting selected item")
                                .as_str();

                            self.viewer.results_pager.paged_item = selection.to_string();

                            self.viewer.list_items(
                                vec![viewer_root.clone().unwrap().to_string()],
                                "request",
                            );
                            self.focus = Focus::Viewer;
                            self.viewer
                                .state
                                .open(vec![viewer_root.unwrap().to_string()]);
                            true
                        }
                    };
                }
                Focus::Viewer => {
                    self.viewer.list_items(path, action);
                }
            },
        }
        true
    }
}
