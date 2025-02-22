use iced::widget::{button, column, scrollable, text};
use iced::{executor, Application, Command, Element, Settings, Theme};
use std::fs;
use std::path::{PathBuf};

fn main() -> iced::Result {
    FileExplorer::run(Settings::default())
}

struct FileExplorer {
    current_path: PathBuf,
    files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeDirectory(PathBuf),
    FilesLoaded(Vec<PathBuf>),
}

impl Application for FileExplorer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: ()) -> (Self, Command<Self::Message>) {
        let start_path = std::env::current_dir().unwrap(); // Dossier initial = dossier courant
        (
            Self {
                current_path: start_path.clone(),
                files: vec![],
            },
            Command::perform(load_files(start_path), Message::FilesLoaded),
        )
    }

    fn title(&self) -> String {
        format!("Explorateur de fichiers - {:?}", self.current_path)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ChangeDirectory(path) => {
                self.current_path = path.clone();
                Command::perform(load_files(path), Message::FilesLoaded)
            }
            Message::FilesLoaded(files) => {
                self.files = files;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let mut file_list = vec![];

        // Bouton pour remonter d'un dossier
        if let Some(parent) = self.current_path.parent() {
            file_list.push(
                button(text(".. (Dossier parent)"))
                    .on_press(Message::ChangeDirectory(parent.to_path_buf()))
                    .into(),
            );
        }

        // Affichage des fichiers/dossiers
        for entry in &self.files {
            let file_name = entry.file_name().unwrap().to_string_lossy().to_string();
            let btn = if entry.is_dir() {
                button(text(format!("📁 {}", file_name)))
                    .on_press(Message::ChangeDirectory(entry.clone()))
            } else {
                button(text(format!("📄 {}", file_name))) // Non cliquable pour les fichiers
            };
            file_list.push(btn.into());
        }

        scrollable(column(file_list)).into()
    }
}

async fn load_files(path: PathBuf) -> Vec<PathBuf> {
    fs::read_dir(&path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_else(|_| vec![]) // En cas d'erreur, retourne une liste vide
}
