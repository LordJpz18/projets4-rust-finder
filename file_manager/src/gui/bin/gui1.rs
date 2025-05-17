use crate::research::{
    search::build_file_tree, search::find_file_btree, search::find_recent_files,
    search::is_application,
};
use iced::window;
use iced::{
    widget::{button, column, text, text_input, Container, Row},
    Alignment, Element, Length, Sandbox, Settings,
};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
struct FileExplorer {
    search_query: String,
    current_view: String,
    file_tree: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    RecentClicked,
    AppsClicked,
    DownloadsClicked,
    DocumentsClicked,
}

impl Sandbox for FileExplorer {
    type Message = Message;

    fn new() -> Self {
        Self {
            file_tree: build_file_tree(), // Initialisation de l'arbre au démarrage
            ..Self::default()
        }
    }

    fn title(&self) -> String {
        String::from("File Manager")
    }

    fn view(&self) -> Element<Message> {
        let search_bar = text_input("Search...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .width(Length::Fill);

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

        let content = text(&self.current_view).size(20);

        let layout = Row::new()
            .push(
                Container::new(sidebar)
                    .width(Length::Units(200))
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
                self.search_query = query.clone();
                let results = find_file_btree(&query, &self.file_tree);
                self.current_view = if results.is_empty() {
                    format!("Aucun résultat pour '{}'", query)
                } else {
                    results.join("\n")
                };
            }
            Message::RecentClicked => {
                let home = dirs::home_dir().unwrap_or_else(|| Path::new("/").to_path_buf());
                let recent_files = find_recent_files(&home);
                self.current_view = recent_files
                    .into_iter()
                    .map(|(path, _)| path.to_string_lossy().into_owned())
                    .collect::<Vec<String>>()
                    .join("\n");
                if self.current_view.is_empty() {
                    self.current_view = String::from("Aucun fichier récent trouvé");
                }
            }
            Message::AppsClicked => {
                let apps: Vec<String> = self
                    .file_tree
                    .values()
                    .filter(|path| is_application(Path::new(path)))
                    .cloned()
                    .collect();
                self.current_view = if apps.is_empty() {
                    String::from("Aucune application trouvée")
                } else {
                    apps.join("\n")
                };
            }
            Message::DownloadsClicked => {
                let downloads_dir = dirs::download_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap().join("Downloads"));
                let downloads: Vec<String> = self
                    .file_tree
                    .values()
                    .filter(|path| Path::new(path).starts_with(&downloads_dir))
                    .cloned()
                    .collect();
                self.current_view = if downloads.is_empty() {
                    String::from("Aucun téléchargement trouvé")
                } else {
                    downloads.join("\n")
                };
            }
            Message::DocumentsClicked => {
                let docs_dir = dirs::document_dir()
                    .unwrap_or_else(|| dirs::home_dir().unwrap().join("Documents"));
                let documents: Vec<String> = self
                    .file_tree
                    .values()
                    .filter(|path| Path::new(path).starts_with(&docs_dir))
                    .cloned()
                    .collect();
                self.current_view = if documents.is_empty() {
                    String::from("Aucun document trouvé")
                } else {
                    documents.join("\n")
                };
            }
        }
    }
}
