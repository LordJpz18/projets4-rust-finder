use iced::alignment::Horizontal;
use iced::widget::{button, column, container, image, row, scrollable, text, text_input};
use iced::{executor, Application, Command, Element, Font, Length, Settings, Theme};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use strsim::levenshtein;
use walkdir::WalkDir;
use std::os::unix::fs::PermissionsExt;

fn main() -> iced::Result {
    FileExplorer::run(Settings::default())
}

struct FileExplorer {
    current_path: PathBuf,
    files: Vec<FileEntry>,
    directory_structure: Vec<DirectoryEntry>,
    path_input: String,
    selected_file: Option<PathBuf>,
    file_preview: Option<FilePreview>,
    system_locations: Vec<SystemLocation>,
    file_name_input: String,
    confirm_delete: Option<PathBuf>,
    clipboard: Option<ClipboardItem>,
    show_hidden: bool,
    search_input: String,
    search_results: Vec<SearchResult>,
    search_cache: HashMap<String, (Vec<SearchResult>, Instant)>,
    file_index: HashMap<String, Vec<PathBuf>>,
    last_index_update: Option<Instant>,
    context_menu: Option<(ContextMenuType, PathBuf)>,
    rename_input: String,
    is_renaming: bool,
    path_to_rename: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ClipboardItem {
    path: PathBuf,
    operation: ClipboardOperation,
}

#[derive(Debug, Clone, PartialEq)]
enum ClipboardOperation {
    Copy,
    Cut,
}

#[derive(Debug, Clone)]
struct SystemLocation {
    name: String,
    path: PathBuf,
    location_type: SystemLocationType,
}

#[derive(Debug, Clone, PartialEq)]
enum SystemLocationType {
    Disk,
    UserFolder,
}

#[derive(Debug, Clone, PartialEq)]
enum FileType {
    Directory,
    Code,
    PDF,
    Audio,
    Image,
    Video,
    Text,
    Word,
    Excel,
    PowerPoint,
    Other,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
    file_type: FileType,
    permissions: String,
}

#[derive(Debug, Clone)]
struct DirectoryEntry {
    path: PathBuf,
    name: String,
    depth: usize,
    expanded: bool,
}

#[derive(Debug, Clone)]
struct SearchResult {
    path: PathBuf,
    score: f64,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeDirectory(PathBuf),
    FilesLoaded(Vec<FileEntry>),
    DirectoryStructureLoaded(Vec<DirectoryEntry>),
    SystemLocationsLoaded(Vec<SystemLocation>),
    PathInputChanged(String),
    NavigateToPath,
    FileSelected(PathBuf),
    ToggleDirectory(usize),
    FilePreviewLoaded(Option<FilePreview>),
    OpenFile(PathBuf),
    CreateFile,
    CreateDirectory,
    FileNameInputChanged(String),
    DeleteFile(PathBuf),
    ConfirmDeleteFile(PathBuf),
    CancelDelete,
    DeleteConfirmed(PathBuf),
    CopyFile(PathBuf),
    CutFile(PathBuf),
    PasteFile,
    ToggleHiddenFiles,
    SearchInputChanged(String),
    ClearSearch,
    ShowContextMenu(Option<PathBuf>, ContextMenuType),
    HideContextMenu,
    RenameFile(PathBuf),
    RenameInputChanged(String),
    ConfirmRename,
    CancelRename,
    RenameInputChanged1(String),
    RenameFile1(PathBuf),
}

#[derive(Debug, Clone)]
enum FilePreview {
    Image(image::Handle),
    Text(String),
    Pdf(String),
    Other(String),
}

#[derive(Debug, Clone)]
enum SearchType {
    Fuzzy,
    Regex,
}

#[derive(Debug, Clone)]
enum ContextMenuType {
    File,
    Directory,
    Empty,
}

impl Application for FileExplorer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: ()) -> (Self, Command<Self::Message>) {
        let start_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        (
            Self {
                current_path: start_path.clone(),
                files: vec![],
                directory_structure: vec![],
                path_input: start_path.to_string_lossy().to_string(),
                selected_file: None,
                file_preview: None,
                system_locations: vec![],
                file_name_input: String::new(),
                confirm_delete: None,
                clipboard: None,
                show_hidden: false,
                search_input: String::new(),
                search_results: Vec::new(),
                search_cache: HashMap::new(),
                file_index: HashMap::new(),
                last_index_update: None,
                context_menu: None,
                rename_input: String::new(),
                is_renaming: false,
                path_to_rename: None,
            },
            Command::batch(vec![
                Command::perform(load_files(start_path.clone(), false), Message::FilesLoaded),
                Command::perform(
                    load_directory_structure(start_path, false),
                    Message::DirectoryStructureLoaded,
                ),
                Command::perform(load_system_locations(), Message::SystemLocationsLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Find-It")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ChangeDirectory(path) => {
                self.current_path = path.clone();
                self.path_input = path.to_string_lossy().to_string();
                self.selected_file = None;
                self.file_preview = None;

                Command::batch(vec![
                    Command::perform(
                        load_files(path.clone(), self.show_hidden),
                        Message::FilesLoaded,
                    ),
                    Command::perform(
                        load_directory_structure(path, self.show_hidden),
                        Message::DirectoryStructureLoaded,
                    ),
                ])
            }
            Message::FilesLoaded(files) => {
                self.files = files;
                Command::none()
            }
            Message::DirectoryStructureLoaded(dirs) => {
                self.directory_structure = dirs;
                Command::none()
            }
            Message::SystemLocationsLoaded(locations) => {
                self.system_locations = locations;
                Command::none()
            }
            Message::PathInputChanged(path) => {
                self.path_input = path;
                Command::none()
            }
            Message::NavigateToPath => {
                let path = PathBuf::from(&self.path_input);
                if path.exists() && path.is_dir() {
                    self.current_path = path.clone();
                    self.selected_file = None;
                    self.file_preview = None;

                    Command::perform(load_files(path, self.show_hidden), Message::FilesLoaded)
                } else {
                    Command::none()
                }
            }
            Message::FileSelected(path) => {
                self.selected_file = Some(path.clone());

                Command::perform(load_file_preview(path), Message::FilePreviewLoaded)
            }
            Message::ToggleDirectory(index) => {
                if index < self.directory_structure.len() {
                    self.directory_structure[index].expanded =
                        !self.directory_structure[index].expanded;

                    let root_path = self.directory_structure[0].path.clone();
                    Command::perform(
                        load_directory_structure(root_path, self.show_hidden),
                        Message::DirectoryStructureLoaded,
                    )
                } else {
                    Command::none()
                }
            }
            Message::FilePreviewLoaded(preview) => {
                self.file_preview = preview;
                Command::none()
            }
            Message::OpenFile(path) => {
                #[cfg(target_os = "windows")]
                {
                    std::process::Command::new("cmd")
                        .args(&["/C", "start", "", path.to_string_lossy().as_ref()])
                        .spawn()
                        .ok();
                }

                #[cfg(target_os = "macos")]
                {
                    std::process::Command::new("open")
                        .arg(path.to_string_lossy().as_ref())
                        .spawn()
                        .ok();
                }

                #[cfg(target_os = "linux")]
                {
                    std::process::Command::new("xdg-open")
                        .arg(path.to_string_lossy().as_ref())
                        .spawn()
                        .ok();
                }

                Command::none()
            }
            Message::FileNameInputChanged(new_name) => {
                self.file_name_input = new_name;
                Command::none()
            }
            Message::CreateFile => {
                let path = self.current_path.join(&self.file_name_input);
                std::fs::write(&path, "").ok();
                self.file_name_input.clear();
                Command::perform(
                    load_files(self.current_path.clone(), self.show_hidden),
                    Message::FilesLoaded,
                )
            }

            Message::CreateDirectory => {
                let path = self.current_path.join(&self.file_name_input);
                std::fs::create_dir_all(&path).ok();
                self.file_name_input.clear();
                Command::perform(
                    load_files(self.current_path.clone(), self.show_hidden),
                    Message::FilesLoaded,
                )
            }
            Message::DeleteFile(path) => {
                let _ = std::fs::remove_file(&path);
                self.selected_file = None;
                self.file_preview = None;

                Command::perform(
                    load_files(self.current_path.clone(), self.show_hidden),
                    Message::FilesLoaded,
                )
            }
            Message::ConfirmDeleteFile(path) => {
                self.confirm_delete = Some(path);
                Command::none()
            }

            Message::CancelDelete => {
                self.confirm_delete = None;
                Command::none()
            }

            Message::DeleteConfirmed(path) => {
                let _ = std::fs::remove_file(&path);
                self.confirm_delete = None;
                self.selected_file = None;
                self.file_preview = None;

                Command::perform(
                    load_files(self.current_path.clone(), self.show_hidden),
                    Message::FilesLoaded,
                )
            }

            Message::CopyFile(path) => {
                self.clipboard = Some(ClipboardItem {
                    path,
                    operation: ClipboardOperation::Copy,
                });
                Command::none()
            }

            Message::CutFile(path) => {
                self.clipboard = Some(ClipboardItem {
                    path,
                    operation: ClipboardOperation::Cut,
                });
                Command::none()
            }

            Message::PasteFile => {
                if let Some(clipboard_item) = &self.clipboard {
                    let source_path = &clipboard_item.path;
                    let file_name = source_path.file_name().unwrap_or_default();
                    let dest_path = self.current_path.join(file_name);

                    match clipboard_item.operation {
                        ClipboardOperation::Copy => {
                            if source_path.is_file() {
                                let _ = std::fs::copy(source_path, &dest_path);
                            } else if source_path.is_dir() {
                                let _ = copy_dir_recursive(source_path, &dest_path);
                            }
                        }
                        ClipboardOperation::Cut => {
                            //VIPER
                            let _ = std::fs::rename(source_path, &dest_path);
                            self.clipboard = None;
                        }
                    }

                    return Command::perform(
                        load_files(self.current_path.clone(), self.show_hidden),
                        Message::FilesLoaded,
                    );
                }
                Command::none()
            }
            Message::ToggleHiddenFiles => {
                self.show_hidden = !self.show_hidden;
                Command::batch(vec![
                    Command::perform(
                        load_files(self.current_path.clone(), self.show_hidden),
                        Message::FilesLoaded,
                    ),
                    Command::perform(
                        load_directory_structure(self.current_path.clone(), self.show_hidden),
                        Message::DirectoryStructureLoaded,
                    ),
                ])
            }
            Message::SearchInputChanged(input) => {
                self.search_input = input;
                if !self.search_input.is_empty() {
                    let search_terms: Vec<String> = self
                        .search_input
                        .to_lowercase()
                        .split_whitespace()
                        .map(String::from)
                        .collect();

                    self.search_results = self.search_files(&search_terms);
                } else {
                    self.search_results.clear();
                }
                Command::none()
            }
            Message::ClearSearch => {
                self.search_input.clear();
                self.search_results.clear();
                Command::none()
            }
            Message::ShowContextMenu(path, menu_type) => {
                self.context_menu = Some((menu_type, path.unwrap_or_else(|| self.current_path.clone())));
                Command::none()
            }
            Message::HideContextMenu => {
                self.context_menu = None;
                Command::none()
            }
            Message::RenameFile(path) => {
                self.context_menu = None;
                self.is_renaming = true;
                self.path_to_rename = Some(path.clone());
                self.rename_input = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                Command::none()
            }
            Message::RenameInputChanged(new_name) => {
                self.rename_input = new_name;
                Command::none()
            }
            Message::ConfirmRename => {
                if let Some(path) = &self.path_to_rename {
                    if let Some(parent) = path.parent() {
                        let new_path = parent.join(&self.rename_input);
                        let _ = std::fs::rename(path, &new_path);
                    }
                }
                self.is_renaming = false;
                self.rename_input.clear();
                self.path_to_rename = None;
                self.context_menu = None;
                Command::batch(vec![
                    Command::perform(
                        load_files(self.current_path.clone(), self.show_hidden),
                        Message::FilesLoaded,
                    ),
                    Command::perform(
                        load_directory_structure(self.current_path.clone(), self.show_hidden),
                        Message::DirectoryStructureLoaded,
                    ),
                ])
            }
            Message::CancelRename => {
                self.is_renaming = false;
                self.rename_input.clear();
                self.path_to_rename = None;
                self.context_menu = None;
                Command::none()
            }
            Message::RenameInputChanged1(input) => {
                self.rename_input = input;
                Command::none()
            }
            Message::RenameFile1(old_path) => {
                if let Some(parent) = old_path.parent() {
                let new_path = parent.join(&self.rename_input);
                if let Err(err) = fs::rename(&old_path, &new_path) {
                    eprintln!("Rename error: {}", err);
                } else {
                    self.rename_input.clear();
                    self.selected_file = Some(new_path.clone());

                    return Command::perform(
                        load_files(self.current_path.clone(), self.show_hidden),
                        Message::FilesLoaded,
                    );
                }
            }
            Command::none()
        }
        }
    }
    fn view(&self) -> Element<Message> {
        let content = if self.is_renaming {
            self.view_rename_dialog()
        } else {
            self.view_main_content()
        };

        let navigation_bar = row![
            text_input("Enter path...", &self.path_input)
                .on_input(Message::PathInputChanged)
                .on_submit(Message::NavigateToPath)
                .width(Length::FillPortion(2)),
            button(text("Go"))
                .on_press(Message::NavigateToPath)
                .padding(2),
            button(text(if self.show_hidden {
                "Hide Hidden Files"
            } else {
                "Show Hidden Files"
            }))
            .on_press(Message::ToggleHiddenFiles)
            .padding(2),
            text_input("Search...", &self.search_input)
                .on_input(Message::SearchInputChanged)
                .width(Length::FillPortion(2)),
            button("Erase").on_press(Message::ClearSearch).padding(2),
        ]
        .spacing(5)
        .padding(5);

        column![
            navigation_bar,
            container(content).height(Length::Fill).width(Length::Fill)
        ]
        .into()
    }
    
}

impl FileExplorer {
    fn view_rename_dialog(&self) -> Element<Message> {
        let rename_input = text_input("Enter new name", &self.rename_input)
            .on_input(Message::RenameInputChanged)
            .on_submit(Message::ConfirmRename)
            .padding(10);

        let buttons = row![
            button("Confirm").on_press(Message::ConfirmRename),
            button("Cancel").on_press(Message::CancelRename)
        ]
        .spacing(10);

        container(
            column![
                text("Rename").size(20),
                rename_input,
                buttons
            ]
            .spacing(10)
            .padding(20)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn view_context_menu(&self) -> Element<Message> {
        if let Some((menu_type, path)) = &self.context_menu {
            let menu_items = match menu_type {
                ContextMenuType::File => {
                    column![
                        button("Open")
                            .on_press(Message::OpenFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Rename")
                            .on_press(Message::RenameFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Copy")
                            .on_press(Message::CopyFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Cut")
                            .on_press(Message::CutFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Delete")
                            .on_press(Message::ConfirmDeleteFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill)
                    ]
                }
                ContextMenuType::Directory => {
                    column![
                        button("Open")
                            .on_press(Message::ChangeDirectory(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Rename")
                            .on_press(Message::RenameFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Copy")
                            .on_press(Message::CopyFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Cut")
                            .on_press(Message::CutFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("Delete")
                            .on_press(Message::ConfirmDeleteFile(path.clone()))
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill)
                    ]
                }
                ContextMenuType::Empty => {
                    column![
                        button("New File")
                            .on_press(Message::CreateFile)
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        button("New Directory")
                            .on_press(Message::CreateDirectory)
                            .style(iced::theme::Button::Secondary)
                            .width(Length::Fill),
                        if self.clipboard.is_some() {
                            button("Paste")
                                .on_press(Message::PasteFile)
                                .style(iced::theme::Button::Secondary)
                                .width(Length::Fill)
                        } else {
                            button("")
                                .style(iced::theme::Button::Secondary)
                                .width(Length::Fill)
                        }
                    ]
                }
            };

            container(
                column![
                    menu_items.spacing(2),
                    button("Cancel")
                        .on_press(Message::HideContextMenu)
                        .style(iced::theme::Button::Secondary)
                        .width(Length::Fill)
                ]
                .spacing(2)
                .padding(2)
            )
            .style(iced::theme::Container::Box)
            .width(Length::Fixed(150.0))
            .into()
        } else {
            text("").into()
        }
    }

    fn view_main_content(&self) -> Element<Message> {
        let content: Element<Message> = if !self.search_results.is_empty() {
            let mut results_list = vec![];

            for result in &self.search_results {
                let path = result.path.clone();
                let score = result.score;
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let parent = path
                    .parent()
                    .and_then(|p| p.strip_prefix(&self.current_path).ok())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let file_type = get_file_type(&path);
                let icon_path = match file_type {
                    FileType::Directory => "icons/folder.png",
                    FileType::Code => "icons/code.png",
                    FileType::PDF => "icons/pdf.png",
                    FileType::Audio => "icons/audio.png",
                    FileType::Image => "icons/image.png",
                    FileType::Video => "icons/video.png",
                    FileType::Text => "icons/file_generic.png",
                    FileType::Word => "icons/word.png",
                    FileType::Excel => "icons/excel.png",
                    FileType::PowerPoint => "icons/pptx.png",
                    FileType::Other => "icons/file_generic.png",
                };

                let is_selected = self.selected_file.as_ref().map_or(false, |p| p == &path);

                let message = if file_type == FileType::Directory {
                    Message::ChangeDirectory(path.clone())
                } else {
                    Message::FileSelected(path.clone())
                };

                let row_content =
                    row![
                        image(image::Handle::from_path(icon_path))
                            .width(Length::Fixed(20.0))
                            .height(Length::Fixed(20.0)),
                        column![
                            row![
                                text(name).width(Length::Fill).style(if is_selected {
                                    iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.0))
                                } else {
                                    iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.0, 0.0))
                                }),
                                text(format!(" ({:.0}%)", score * 100.0)).size(12).style(
                                    iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))
                                )
                            ],
                            if !parent.is_empty() {
                                text(format!("📍 {}", parent)).size(12).style(
                                    iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                )
                            } else {
                                text("")
                            }
                        ]
                        .spacing(2)
                    ]
                    .spacing(10)
                    .padding(5);

                let btn = button(row_content)
                    .on_press(message)
                    .style(if is_selected {
                        iced::theme::Button::Positive
                    } else {
                        iced::theme::Button::Secondary
                    })
                    .width(Length::Fill);

                results_list.push(btn.into());
            }

            // Ajout de la zone de détails pour le fichier sélectionné dans les résultats
            let details = if self.selected_file.is_some() && !self.search_results.is_empty() {
                self.view_file_details()
            } else {
                container(text("Aucun fichier sélectionné")).into()
            };

            row![
                container(
                    scrollable(container(column(results_list).spacing(5)).padding(10))
                        .height(Length::Fill)
                )
                .width(Length::FillPortion(2)),
                container(details)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
            ]
            .spacing(10)
            .into()
        } else {
            row![
                container(self.view_directory_structure()).width(Length::FillPortion(1)),
                container(self.view_file_list()).width(Length::FillPortion(2)),
                container(self.view_file_details()).width(Length::FillPortion(1))
            ]
            .spacing(10)
            .into()
        };

        column![
            container(content).height(Length::Fill).width(Length::Fill)
        ]
        .into()
    }

    fn view_directory_structure(&self) -> Element<Message> {
        let tree_title = text("Quick Access")
            .size(16)
            .horizontal_alignment(Horizontal::Center);
        let mut dir_list = vec![tree_title.into()];

        let file_name_input = text_input("Enter file name", &self.file_name_input)
            .on_input(Message::FileNameInputChanged)
            .on_submit(Message::CreateFile)
            .padding(10);

        let create_buttons = row![
            button("Create File").on_press(Message::CreateFile),
            button("Create Directory").on_press(Message::CreateDirectory)
        ]
        .spacing(10);

        dir_list.push(
            container(file_name_input)
                .padding(5)
                .width(Length::Fill)
                .into(),
        );
        dir_list.push(
            container(create_buttons)
                .padding(5)
                .width(Length::Fill)
                .into(),
        );

        if let Some(selected_path) = &self.selected_file {
            let clipboard_buttons = column![
                row![
                    button("Copy").on_press(Message::CopyFile(selected_path.clone())),
                    button("Cut").on_press(Message::CutFile(selected_path.clone()))
                ]
                .spacing(5),
                if self.clipboard.is_some() {
                    button("Paste")
                        .on_press(Message::PasteFile)
                        .style(iced::theme::Button::Positive)
                } else {
                    button("").style(iced::theme::Button::Secondary)
                }
            ]
            .spacing(5);

            dir_list.push(
                container(clipboard_buttons)
                    .padding(5)
                    .width(Length::Fill)
                    .into(),
            );
        } else if self.clipboard.is_some() {
            let paste_button = button("Paste")
                .on_press(Message::PasteFile)
                .style(iced::theme::Button::Positive);

            dir_list.push(
                container(paste_button)
                    .padding(5)
                    .width(Length::Fill)
                    .into(),
            );
        }

        if let Some(clipboard_item) = &self.clipboard {
            let operation_text = match clipboard_item.operation {
                ClipboardOperation::Copy => "Copied:",
                ClipboardOperation::Cut => "Cut:",
            };
            let file_name = clipboard_item
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            dir_list.push(
                container(
                    column![text(operation_text).size(12), text(file_name).size(10)].spacing(2),
                )
                .padding(5)
                .width(Length::Fill)
                .into(),
            );
        }

        for location in &self.system_locations {
            /*let icon = match location.location_type {
                SystemLocationType::Disk => "💾 ",
                SystemLocationType::UserFolder => "📁 ",
            };*/

            let content = row![text(format!("{}",location.name))].spacing(5);

            let btn = button(content)
                .on_press(Message::ChangeDirectory(location.path.clone()))
                .width(Length::Fill);

            dir_list.push(container(btn).padding(5).width(Length::Fill).into());
        }

        dir_list.push(
            container(
                text("Current Directory")
                    .size(16)
                    .horizontal_alignment(Horizontal::Center),
            )
            .padding([20, 0, 5, 0])
            .into(),
        );

        for (index, dir) in self.directory_structure.iter().enumerate() {
            let should_show = true;

            if should_show {
                let indent = dir.depth * 20;

                let toggle_icon = if dir.expanded { "▼ " } else { "▶ " };

                let content = row![
                    container(text(toggle_icon)).width(Length::Fixed(20.0)),
                    text(format!("{}", dir.name))
                ]
                .spacing(5);

                let btn = button(content)
                    .on_press(Message::ChangeDirectory(dir.path.clone()))
                    .width(Length::Fill);

                dir_list.push(
                    container(btn)
                        .padding(5)
                        .width(Length::Fill)
                        .padding([0, 0, 0, indent as u16])
                        .into(),
                );
            }
        }

        scrollable(
            container(column(dir_list).spacing(2))
                .padding(10)
                .height(Length::Fill),
        )
        .into()
    }

    fn view_file_list(&self) -> Element<Message> {
        let mut file_list = vec![];

        file_list.push(
            row![
                text("Name").width(Length::FillPortion(3)),
                text("Size").width(Length::FillPortion(1)),
                text("Modified").width(Length::FillPortion(2)),
                text("").width(Length::Fixed(30.0)) // Espace pour le bouton de menu
            ]
            .spacing(10)
            .padding(5)
            .into(),
        );

        if let Some(parent) = self.current_path.parent() {
            file_list.push(
                row![
                    button(
                        row![
                            image(image::Handle::from_path("icons/folder.png"))
                                .width(Length::Fixed(20.0))
                                .height(Length::Fixed(20.0)),
                            text(".. (Parent Directory)").width(Length::FillPortion(3)),
                            text("").width(Length::FillPortion(1)),
                            text("").width(Length::FillPortion(2)),
                            button("⋮")
                                .on_press(Message::ShowContextMenu(Some(parent.to_path_buf()), ContextMenuType::Directory))
                                .width(Length::Fixed(30.0))
                        ]
                        .spacing(10),
                    )
                    .on_press(Message::ChangeDirectory(parent.to_path_buf()))
                    .width(Length::Fill)
                ]
                .into(),
            );
        }

        for entry in &self.files {
            let file_name = entry
                .path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                .to_string_lossy()
                .to_string();

            let size_text = format_size(entry.size);

            let modified_text = entry
                .modified
                .map(|time| format_time(time))
                .unwrap_or_else(|| String::from("-"));

            let is_selected = self
                .selected_file
                .as_ref()
                .map_or(false, |p| p == &entry.path);

            let icon_path = match entry.file_type {
                FileType::Directory => "icons/folder.png",
                FileType::Code => "icons/code.png",
                FileType::PDF => "icons/pdf.png",
                FileType::Audio => "icons/audio.png",
                FileType::Image => "icons/image.png",
                FileType::Video => "icons/video.png",
                FileType::Text => "icons/file_generic.png",
                FileType::Word => "icons/word.png",
                FileType::Excel => "icons/excel.png",
                FileType::PowerPoint => "icons/pptx.png",
                FileType::Other => "icons/file_generic.png",
            };

            let menu_button = button("≡")
                .on_press(Message::ShowContextMenu(Some(entry.path.clone()), 
                    if entry.file_type == FileType::Directory {
                        ContextMenuType::Directory
                    } else {
                        ContextMenuType::File
                    }
                ))
                .style(iced::theme::Button::Secondary)
                .width(Length::Fixed(30.0));

            let row_content = row![
                image(image::Handle::from_path(icon_path))
                    .width(Length::Fixed(20.0))
                    .height(Length::Fixed(20.0)),
                text(file_name)
                    .width(Length::FillPortion(3))
                    .style(if is_selected {
                        iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.0))
                    } else {
                        iced::theme::Text::Default
                    }),
                text(size_text)
                    .width(Length::FillPortion(1))
                    .style(if is_selected {
                        iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.0))
                    } else {
                        iced::theme::Text::Default
                    }),
                text(modified_text)
                    .width(Length::FillPortion(2))
                    .style(if is_selected {
                        iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.6, 0.0))
                    } else {
                        iced::theme::Text::Default
                    }),
                if self.context_menu.as_ref().map_or(false, |(_, p)| p == &entry.path) {
                    container(self.view_context_menu())
                        .style(iced::theme::Container::Box)
                        .width(Length::Fixed(150.0))
                } else {
                    container(menu_button)
                        .width(Length::Fixed(30.0))
                }
            ]
            .spacing(10);

            let btn = if entry.file_type == FileType::Directory {
                button(row_content)
                    .on_press(Message::ChangeDirectory(entry.path.clone()))
                    .width(Length::Fill)
            } else {
                button(row_content)
                    .on_press(Message::FileSelected(entry.path.clone()))
                    .style(iced::theme::Button::Text)
                    .width(Length::Fill)
            };

            file_list.push(btn.into());
        }

        scrollable(
            container(column(file_list).spacing(5))
                .padding(10)
                .height(Length::Fill),
        )
        .into()
    }

    fn view_file_details(&self) -> Element<Message> {
        let mut details = vec![];
        if let Some(selected_path) = &self.selected_file {
            let rename_input = text_input("New name", &self.rename_input)
                .on_input(Message::RenameInputChanged1)
                .padding(10)
                .width(Length::Fill);

            let rename_button = button("Rename")
                .on_press(Message::RenameFile1(selected_path.clone()))
                .padding(8);

            details.push(
                column![
                    text("Rename the file").size(16),
                    rename_input,
                    rename_button
                ]
                .spacing(10)
                .into(),
            );
            details.push(
                text("File Details")
                    .size(16)
                    .horizontal_alignment(Horizontal::Center)
                    .into(),
            );
        }

        if let Some(selected_path) = &self.selected_file {
            let file_name = selected_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                .to_string_lossy();

            let extension = selected_path
                .extension()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_string_lossy()
                .to_string();

            details.push(text(format!("Name: {}", file_name)).into());
            details.push(text(format!("Type: {}", extension.to_uppercase())).into());

            if let Ok(metadata) = fs::metadata(selected_path) {
                details.push(text(format!("Size: {}", format_size(metadata.len()))).into());

                if let Ok(time) = metadata.modified() {
                    details.push(text(format!("Modified: {}", format_time(time))).into());
                }
                let permissions = get_permissions_string(&metadata);
                details.push(text(format!("File Permissions: {}", permissions)).into());
            }

            if let Some(path) = &self.confirm_delete {
                details.push(
                    container(
                        column![
                            text("Are you sure you want to delete this file ?").size(16),
                            text(path.file_name().unwrap_or_default().to_string_lossy()).size(14),
                            row![
                                button("Delete")
                                    .on_press(Message::DeleteConfirmed(path.clone()))
                                    .style(iced::theme::Button::Destructive),
                                button("Cancel").on_press(Message::CancelDelete)
                            ]
                            .spacing(10)
                        ]
                        .spacing(10)
                        .padding(10),
                    )
                    .style(iced::theme::Container::Box)
                    .width(Length::Fill)
                    .into(),
                );
            } 
            if let Some(file_entry) = self.files.iter().find(|f| f.path == *selected_path) {
                if file_entry.permissions.contains("r") {
                    details.push(
                        button(text("Open File"))
                            .on_press(Message::OpenFile(selected_path.clone()))
                            .padding(5)
                            .into(),
                    );
                } else {
                    details.push(
                        text("You don't have permission to read this file.")
                            .size(14)
                            .style(iced::theme::Text::Color([0.8, 0.2, 0.2].into()))
                            .into(),
                    );
                }

                details.push(
                    button(text("Delete file"))
                        .on_press(Message::ConfirmDeleteFile(selected_path.clone()))
                        .padding(5)
                        .style(iced::theme::Button::Destructive)
                        .into(),
                );
            }

            details.push(text("Preview:").size(16).into());

            if let Some(preview) = &self.file_preview {
                match preview {
                    FilePreview::Image(handle) => {
                        details.push(
                            container(image(handle.clone()).width(Length::Fill))
                                .height(Length::Fixed(200.0))
                                .width(Length::Fill)
                                .center_x()
                                .center_y()
                                .into(),
                        );
                    }
                    FilePreview::Text(content) => {
                        let preview_text = if content.len() > 500 {
                            format!("{}...", &content[..500])
                        } else {
                            content.clone()
                        };

                        details.push(
                            scrollable(text(preview_text).width(Length::Fill))
                                .height(Length::Fixed(200.0))
                                .into(),
                        );
                    }
                    FilePreview::Pdf(info) => {
                        details.push(text(info).into());
                    }
                    FilePreview::Other(info) => {
                        details.push(text(info).into());
                    }
                }
            }
        } else {
            details.push(text("No file selected").into());
        }

        scrollable(
            container(column(details).spacing(10))
                .padding(10)
                .height(Length::Fill),
        )
        .into()
    }

    fn update_file_index(&mut self) {
        let now = Instant::now();
        if let Some(last_update) = self.last_index_update {
            if now.duration_since(last_update) < Duration::from_secs(300) {
                return;
            }
        }

        self.file_index.clear();

        let ignored_patterns = [
            "/.git/",
            "/node_modules/",
            "/target/",
            "/build/",
            "/dist/",
            "/.idea/",
            "/.vscode/",
            "github",
            "GitHub",
            "GITHUB",
            "github.app",
            "GitHub.app",
            "GITHUB.APP",
            "github-desktop",
            "GitHub Desktop",
            "GITHUB DESKTOP",
            "github-desktop.app",
            "GitHub Desktop.app",
            "GITHUB DESKTOP.APP",
            ".app/",
            ".exe/",
            ".dmg/",
            ".pkg/",
            "SublimeText.app/",
            "Visual Studio Code.app/",
            "Xcode.app/",
            "Chrome.app/",
            "Firefox.app/",
            "Safari.app/",
            "Microsoft Office/",
            "Adobe/",
            "JetBrains/",
            "IntelliJ/",
            "WebStorm/",
            "PhpStorm/",
            "PyCharm/",
            "Android Studio/",
            "Eclipse/",
            "NetBeans/",
            "Atom.app/",
            "iTerm.app/",
            "Terminal.app/",
            "Spotify.app/",
            "Discord.app/",
            "Slack.app/",
            "Zoom.app/",
            "Teams.app/",
            "Microsoft Teams.app/",
            "Dropbox.app/",
            "OneDrive.app/",
            "Google Drive.app/",
            "Docker.app/",
            "Postman.app/",
            "MAMP.app/",
            "XAMPP.app/",
            "MySQL.app/",
            "PostgreSQL.app/",
            "MongoDB.app/",
            "Redis.app/",
            "Node.js.app/",
            "Python.app/",
            "Java.app/",
            "Ruby.app/",
            "Go.app/",
            "Rust.app/",
            "C++.app/",
            "Visual Studio.app/",
            "Unity.app/",
            "Unreal Engine.app/",
            "Blender.app/",
            "Photoshop.app/",
            "Illustrator.app/",
            "InDesign.app/",
            "Premiere Pro.app/",
            "After Effects.app/",
            "Lightroom.app/",
            "Final Cut Pro.app/",
            "Logic Pro.app/",
            "GarageBand.app/",
            "iTunes.app/",
            "Music.app/",
            "TV.app/",
            "Books.app/",
            "News.app/",
            "Stocks.app/",
            "Weather.app/",
            "Maps.app/",
            "Notes.app/",
            "Reminders.app/",
            "Calendar.app/",
            "Contacts.app/",
            "Mail.app/",
            "Messages.app/",
            "FaceTime.app/",
            "Photos.app/",
            "Preview.app/",
            "QuickTime Player.app/",
            "Siri.app/",
            "System Preferences.app/",
            "Terminal.app/",
            "Activity Monitor.app/",
            "Console.app/",
            "Disk Utility.app/",
            "Time Machine.app/",
            "Migration Assistant.app/",
            "Boot Camp Assistant.app/",
            "Automator.app/",
            "Script Editor.app/",
            "Voice Memos.app/",
            "Home.app/",
            "Shortcuts.app/",
            "Stocks.app/",
            "Voice Memos.app/",
            "Calculator.app/",
            "Dictionary.app/",
            "Font Book.app/",
            "Image Capture.app/",
            "Keychain Access.app/",
            "Migration Assistant.app/",
            "System Information.app/",
            "Terminal.app/",
            "Time Machine.app/",
            "Voice Memos.app/",
            "/.DS_Store",
            "/Thumbs.db",
            "*.tmp",
            "*.log",
            "*.cache",
            "*.swp",
            "*.bak",
            "*.temp",
        ];

        let walker = WalkDir::new(&self.current_path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|entry| {
                let path = entry.path().to_string_lossy().to_lowercase();
                let file_name = entry.file_name().to_string_lossy().to_lowercase();

                let should_ignore = ignored_patterns.iter().any(|pattern| {
                    path.contains(&pattern.to_lowercase())
                        || file_name.contains(&pattern.to_lowercase())
                });

                let is_app_extension = file_name.ends_with(".app")
                    || file_name.ends_with(".exe")
                    || file_name.ends_with(".dmg")
                    || file_name.ends_with(".pkg");

                let is_app_folder = path.contains("/applications/")
                    || path.contains("/program files/")
                    || path.contains("/program files (x86)/")
                    || path.contains("/library/application support/");

                !should_ignore && !is_app_extension && !is_app_folder
            });

        for entry in walker.filter_map(Result::ok) {
            let path = entry.path();
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();

            if !self.show_hidden && file_name.starts_with('.') {
                continue;
            }

            if file_name.ends_with(".app")
                || file_name.ends_with(".exe")
                || file_name.ends_with(".dmg")
                || file_name.ends_with(".pkg")
            {
                continue;
            }

            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.contains("/applications/")
                || path_str.contains("/program files/")
                || path_str.contains("/program files (x86)/")
                || path_str.contains("/library/application support/")
            {
                continue;
            }

            self.file_index
                .entry(file_name.clone())
                .or_insert_with(Vec::new)
                .push(path.to_path_buf());

            for word in file_name.split(|c: char| !c.is_alphanumeric()) {
                if !word.is_empty() && word.len() > 1 {
                    self.file_index
                        .entry(word.to_string())
                        .or_insert_with(Vec::new)
                        .push(path.to_path_buf());
                }
            }
        }

        self.last_index_update = Some(now);
    }

    fn search_files(&mut self, search_terms: &[String]) -> Vec<SearchResult> {
        let cache_key = format!(
            "{}_{}",
            self.current_path.to_string_lossy(),
            search_terms.join(" ")
        );
        if let Some((cached_results, timestamp)) = self.search_cache.get(&cache_key) {
            if Instant::now().duration_since(*timestamp) < Duration::from_secs(60) {
                return cached_results.clone();
            }
        }

        self.update_file_index();

        let mut results = Vec::new();
        let mut seen_paths = HashSet::new();
        let mut path_scores = HashMap::new();

        let important_folders = [
            "Documents",
            "Downloads",
            "Desktop",
            "Pictures",
            "Music",
            "Videos",
            "Projects",
            "Work",
            "Personal",
        ];

        // Détecter si la recherche est une regex
        let search_type = if search_terms.len() == 1
            && search_terms[0].starts_with('/')
            && search_terms[0].ends_with('/')
        {
            SearchType::Regex
        } else {
            SearchType::Fuzzy
        };

        match search_type {
            SearchType::Regex => {
                let pattern = &search_terms[0][1..search_terms[0].len() - 1];
                if let Ok(regex) = Regex::new(pattern) {
                    for (indexed_term, paths) in &self.file_index {
                        if regex.is_match(indexed_term) {
                            for path in paths {
                                if seen_paths.insert(path.clone()) {
                                    let file_name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_lowercase();

                                    let matches: Vec<_> = regex.find_iter(&file_name).collect();
                                    let score = if !matches.is_empty() {
                                        let total_match_length: usize =
                                            matches.iter().map(|m| m.end() - m.start()).sum();
                                        (total_match_length as f64 / file_name.len() as f64)
                                            .min(1.0)
                                    } else {
                                        0.0
                                    };

                                    if score >= 0.3 {
                                        path_scores.insert(path.clone(), score);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SearchType::Fuzzy => {
                for term in search_terms {
                    for (indexed_term, paths) in &self.file_index {
                        let distance = levenshtein(term, indexed_term) as f64;
                        let max_len = term.len().max(indexed_term.len()) as f64;
                        let mut score = 1.0 - (distance / max_len);

                        if indexed_term == term {
                            score *= 1.5;
                        }

                        if let Some(file_name) = paths[0].file_name() {
                            let name = file_name.to_string_lossy().to_lowercase();
                            if important_folders
                                .iter()
                                .any(|&folder| name.contains(folder))
                            {
                                score *= 1.3;
                            }
                        }

                        if score >= 0.3 {
                            for path in paths {
                                if seen_paths.insert(path.clone()) {
                                    let file_name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_lowercase();

                                    let mut total_score = 0.0;
                                    let mut best_score: f64 = 0.0;
                                    let mut match_count = 0;

                                    for search_term in search_terms {
                                        let term_distance =
                                            levenshtein(search_term, &file_name) as f64;
                                        let term_max_len =
                                            search_term.len().max(file_name.len()) as f64;
                                        let mut term_score = 1.0 - (term_distance / term_max_len);

                                        if file_name.starts_with(search_term) {
                                            term_score *= 1.2;
                                        }

                                        if let Some(parent) = path.parent() {
                                            let parent_name = parent
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy()
                                                .to_lowercase();
                                            if important_folders
                                                .iter()
                                                .any(|&folder| parent_name.contains(folder))
                                            {
                                                term_score *= 1.1;
                                            }
                                        }

                                        if file_name.contains(search_term) {
                                            total_score += term_score;
                                            match_count += 1;
                                        }
                                        best_score = best_score.max(term_score);
                                    }

                                    let final_score = if match_count > 0 {
                                        (total_score / match_count as f64) * 0.7
                                            + (best_score * 0.3)
                                    } else {
                                        best_score
                                    };

                                    let final_score = final_score.min(1.0).max(0.0);

                                    if final_score >= 0.3 {
                                        path_scores.insert(path.clone(), final_score);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for (path, score) in path_scores {
            results.push(SearchResult { path, score });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(100);

        self.search_cache
            .insert(cache_key, (results.clone(), Instant::now()));

        results
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn get_permissions_string(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let r;
    let w;
    let x;
    if mode & 0o400 != 0 {
        r = "r";
    } else {
        r = "-";
    }
    if mode & 0o200 != 0 {
        w = "w";
    } else {
        w = "-";
    }
    if mode & 0o100 != 0 {
        x = "x";
    } else {
        x = "-";
    }

    format!("{}{}{}", r, w, x)
}


async fn load_files(path: PathBuf, show_hidden: bool) -> Vec<FileEntry> {
    fs::read_dir(&path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    let path = entry.path();
                    let file_name = path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new(""))
                        .to_string_lossy();

                    if !show_hidden {
                        !file_name.starts_with('.')
                    } else {
                        true
                    }
                })
                .map(|entry| {
                    let path = entry.path();
                    let metadata = fs::metadata(&path).ok();
                    let permissions = metadata
                        .as_ref()
                        .map(get_permissions_string)
                        .unwrap_or_else(|| "---".to_string());

                    FileEntry {
                        path: path.clone(),
                        size: metadata.as_ref().map_or(0, |m| m.len()),
                        modified: metadata.and_then(|m| m.modified().ok()),
                        file_type: get_file_type(&path),
                        permissions,
                    }
                })
                .collect()
        })
        .unwrap_or_else(|_| vec![])
}

async fn load_directory_structure(root: PathBuf, show_hidden: bool) -> Vec<DirectoryEntry> {
    let mut dirs = Vec::new();

    let root_name = root
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("Root"))
        .to_string_lossy()
        .to_string();

    dirs.push(DirectoryEntry {
        path: root.clone(),
        name: root_name,
        depth: 0,
        expanded: true,
    });

    scan_directory(&root, &mut dirs, 0, 3, show_hidden);

    dirs
}

async fn load_system_locations() -> Vec<SystemLocation> {
    let mut locations = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for drive_letter in b'C'..=b'Z' {
            let drive = format!("{}:\\", drive_letter as char);
            let path = PathBuf::from(&drive);

            if path.exists() {
                locations.push(SystemLocation {
                    name: format!("{} Drive", drive_letter as char),
                    path,
                    location_type: SystemLocationType::Disk,
                });
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                        .to_string_lossy()
                        .to_string();

                    locations.push(SystemLocation {
                        name,
                        path,
                        location_type: SystemLocationType::Disk,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/media") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                        .to_string_lossy()
                        .to_string();

                    locations.push(SystemLocation {
                        name,
                        path,
                        location_type: SystemLocationType::Disk,
                    });
                }
            }
        }
    }

    if let Some(home_dir) = dirs_next::home_dir() {
        locations.push(SystemLocation {
            name: "Home".to_string(),
            path: home_dir.clone(),
            location_type: SystemLocationType::UserFolder,
        });

        let common_folders = [
            ("Bureau", "Bureau"),
            ("Documents", "Documents"),
            ("Téléchargements", "Téléchargements"),
            ("Musique", "Musique"),
            ("Images", "Images"),
            ("Vidéos", "Vidéos"),
        ];

        for (name, folder) in common_folders.iter() {
            let path = home_dir.join(folder);
            if path.exists() && path.is_dir() {
                locations.push(SystemLocation {
                    name: name.to_string(),
                    path,
                    location_type: SystemLocationType::UserFolder,
                });
            }
        }
    }

    locations
}

fn scan_directory(
    path: &Path,
    dirs: &mut Vec<DirectoryEntry>,
    depth: usize,
    max_depth: usize,
    show_hidden: bool,
) {
    if depth >= max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                let name = entry_path
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                    .to_string_lossy()
                    .to_string();

                if !show_hidden && name.starts_with('.') {
                    continue;
                }

                dirs.push(DirectoryEntry {
                    path: entry_path.clone(),
                    name,
                    depth: depth + 1,
                    expanded: false,
                });

                scan_directory(&entry_path, dirs, depth + 1, max_depth, show_hidden);
            }
        }
    }
}

async fn load_file_preview(path: PathBuf) -> Option<FilePreview> {
    if !path.is_file() {
        return None;
    }

    let metadata = fs::metadata(&path).ok()?;
    let mode = metadata.permissions().mode();

    if mode & 0o400 == 0{
        return Some(FilePreview::Other("File is not readable".into()));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" => {
            if let Ok(bytes) = fs::read(&path) {
                return Some(FilePreview::Image(image::Handle::from_memory(bytes)));
            }
        }

        "txt" | "md" | "rs" | "toml" | "json" | "yaml" | "yml" | "css" | "html" | "js" | "py" => {
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(FilePreview::Text(content));
            }
        }

        "pdf" => {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            return Some(FilePreview::Pdf(format!(
                "PDF document ({} bytes)\nUse 'Open File' to view",
                size
            )));
        }

        _ => {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            return Some(FilePreview::Other(format!(
                "{} file ({} bytes)\nUse 'Open File' to open",
                extension.to_uppercase(),
                size
            )));
        }
    }

    None
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size < KB {
        format!("{} B", size)
    } else if size < MB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else {
        format!("{:.1} GB", size as f64 / GB as f64)
    }
}

fn format_time(time: SystemTime) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let duration = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    let secs = duration.as_secs();

    let seconds = secs % 60;
    let minutes = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    let days = (secs / 86400) % 30;
    let months = (secs / 2592000) % 12;
    let years = secs / 31536000;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        1970 + years,
        1 + months,
        1 + days,
        hours,
        minutes,
        seconds
    )
}

fn get_file_type(path: &Path) -> FileType {
    if path.is_dir() {
        return FileType::Directory;
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "hpp" | "cs" | "go" | "php"
        | "swift" | "kt" | "html" | "css" | "ml" => FileType::Code,
        "pdf" => FileType::PDF,
        "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" => FileType::Audio,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => FileType::Image,
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => FileType::Video,
        "txt" | "md" | "json" | "xml" | "yaml" | "yml" | "csv" | "log" => FileType::Text,
        "doc" | "docx" => FileType::Word,
        "xls" | "xlsx" | "xlsm" => FileType::Excel,
        "ppt" | "pptx" | "pptm" | "key" => FileType::PowerPoint,
        _ => FileType::Other,
    }
}
