use crate::research::{
    search::build_file_tree, search::find_file_btree, search::find_recent_files,
    search::is_application,
};
use iced::{
    widget::{button, column, text, text_input, Container, Image, Row},
    Alignment, Element, Length, Sandbox,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FileExplorer {
    search_query: String,
    current_files: Vec<FileInfo>,
    file_tree: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct FileIcon {
    path: PathBuf,
}

#[derive(Clone)]
pub struct FileInfo {
    path: String,
    icon: FileIcon,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    RecentClicked,
    AppsClicked,
    DownloadsClicked,
    DocumentsClicked,
}

pub fn get_file_icon(path: &str) -> FileIcon {
    let path_obj = Path::new(path);
    if path_obj.is_dir() {
        FileIcon {
            path: PathBuf::from("icons/dir.png"),
        }
    } else if is_application(path_obj) {
        FileIcon {
            path: PathBuf::from("icons/dir.png"),
        }
    } else {
        FileIcon {
            path: PathBuf::from("icons/file.png"),
        }
    }
}

impl Sandbox for FileExplorer {
    type Message = Message;

    fn new() -> Self {
        let file_tree = build_file_tree();
        println!("file_tree initialisé avec {} entrées", file_tree.len());
        Self {
            file_tree: build_file_tree(),
            ..Self::default()
        }
    }

    fn title(&self) -> String {
        String::from("File Manager")
    }

    fn view(&self) -> Element<Message> {
        let search_bar = text_input("Rechercher...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .width(Length::Fill)
            .size(20);

        let sidebar = column![
            button(text("Récents").size(16))
                .on_press(Message::RecentClicked)
                .padding(10)
                .width(Length::Fill),
            button(text("Applications").size(16))
                .on_press(Message::AppsClicked)
                .padding(10)
                .width(Length::Fill),
            button(text("Téléchargements").size(16))
                .on_press(Message::DownloadsClicked)
                .padding(10)
                .width(Length::Fill),
            button(text("Documents").size(16))
                .on_press(Message::DocumentsClicked)
                .padding(10)
                .width(Length::Fill),
        ]
        .spacing(10)
        .align_items(Alignment::Start);

        let content: Element<Message> = if self.current_files.is_empty() {
            text("Aucun fichier à afficher").size(20).into()
        } else {
            let file_rows: Vec<Element<_>> = self
                .current_files
                .iter()
                .map(|file| {
                    let icon = Image::new(file.icon.path.clone())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0));
                    let path_text = text(&file.path).size(16);
                    Row::new().push(icon).push(path_text).spacing(10).into()
                })
                .collect();
            column(file_rows).spacing(10).into()
        };

        let layout = Row::new()
            .push(
                Container::new(sidebar)
                    .width(Length::Fixed(200.0))
                    .height(Length::Fill)
                    .style(|theme: &iced::Theme| iced::widget::container::Appearance {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            240, 240, 240,
                        ))),
                        ..Default::default()
                    }),
            )
            .push(
                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y(),
            )
            .spacing(20);

        column![search_bar, layout].padding(20).spacing(20).into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SearchChanged(query) => {
                println!("SearchChanged déclenché avec query: '{}'", query);
                self.search_query = query.clone();
                let results = find_file_btree(&query, &self.file_tree);
                println!("Résultats de recherche: {:?}", results);
                self.current_files = results
                    .into_iter()
                    .map(|path| FileInfo {
                        path: path.clone(),
                        icon: get_file_icon(&path),
                    })
                    .collect();
            }
            Message::RecentClicked => {
                let recent_files = find_recent_files();
                self.current_files = recent_files
                    .into_iter()
                    .map(|(path, _)| {
                        let path_str = path.to_string_lossy().into_owned();
                        FileInfo {
                            path: path_str.clone(),
                            icon: get_file_icon(&path_str),
                        }
                    })
                    .collect();
            }
            Message::AppsClicked => {
                self.current_files = self
                    .file_tree
                    .values()
                    .filter(|path| is_application(Path::new(path)))
                    .map(|path| FileInfo {
                        path: path.clone(),
                        icon: get_file_icon(path),
                    })
                    .collect();
            }
            Message::DownloadsClicked => {
                let downloads_dir = dirs::download_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap().join("Downloads"));
                self.current_files = self
                    .file_tree
                    .values()
                    .filter(|path| Path::new(path).starts_with(&downloads_dir))
                    .map(|path| FileInfo {
                        path: path.clone(),
                        icon: get_file_icon(path),
                    })
                    .collect();
            }
            Message::DocumentsClicked => {
                let docs_dir = dirs::document_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap().join("Documents"));
                self.current_files = self
                    .file_tree
                    .values()
                    .filter(|path| Path::new(path).starts_with(&docs_dir))
                    .map(|path| FileInfo {
                        path: path.clone(),
                        icon: get_file_icon(path),
                    })
                    .collect();
            }
        }
    }
}
