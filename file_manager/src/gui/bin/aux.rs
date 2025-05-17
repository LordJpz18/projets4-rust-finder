use crate::research::find_file_btree;
use iced::window;
use iced::{
    widget::{button, column, text, text_input, Container, Row},
    Alignment, Element, Length, Sandbox, Settings,
};
use std::collections::BTreeMap; // Importation de la fonction

#[derive(Default)]
struct FileExplorer {
    search_query: String, // Contenu de la barre de recherche
    current_view: String, // Message affiché selon le bouton cliqué
    file_tree: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String), // Mise à jour de la barre de recherche
    RecentClicked,         // Bouton "Récents"
    AppsClicked,           // Bouton "Applications"
    DownloadsClicked,      // Bouton "Téléchargements"
    DocumentsClicked,      // Bouton "Documents"
}

impl Sandbox for FileExplorer {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("File Manager")
    }

    fn view(&self) -> Element<Message> {
        // Barre de recherche en haut
        let search_bar = text_input("Search...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .width(Length::Fill);

        // Sidebar avec les boutons
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

        // Zone principale avec le message
        let content = text(&self.current_view).size(20);

        // Mise en page : sidebar à gauche, contenu à droite
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

        // Ajouter la barre de recherche au-dessus
        column![search_bar, layout].padding(20).spacing(20).into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                // Appelle ta fonction de recherche ici
                self.current_view = search_files(&self.search_query);
            }
            Message::RecentClicked => {
                self.current_view = String::from("Affichage des fichiers récents...");
                // Logique pour "Récents" à ajouter ici
            }
            Message::AppsClicked => {
                self.current_view = String::from("Affichage des applications...");
                // Logique pour "Applications" à ajouter ici
            }
            Message::DownloadsClicked => {
                self.current_view = String::from("Affichage des téléchargements...");
                // Logique pour "Téléchargements" à ajouter ici
            }
            Message::DocumentsClicked => {
                self.current_view = String::from("Affichage des documents...");
                // Logique pour "Documents" à ajouter ici
            }
        }
    }
}

// Fonction de recherche fictive (à remplacer par la tienne)
fn search_files(query: &str) -> String {
    format!("Résultats pour '{}'", query)
}

fn main() -> iced::Result {
    FileExplorer::run(Settings {
        window: window::Settings {
            size: iced::Size::new(800, 600),
            ..Default::default()
        },
        ..Settings::default()
    })
}
