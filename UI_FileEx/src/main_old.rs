use iced::alignment::Horizontal;
use iced::widget::{button, column, container, image, row, scrollable, text, text_input};
use iced::{executor, Application, Command, Element, Length, Settings, Theme, Color};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct DirectoryEntry {
    path: PathBuf,
    name: String,
    depth: usize,
    expanded: bool,
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
}

#[derive(Debug, Clone)]
enum FilePreview {
    Image(image::Handle),
    Text(String),
    Pdf(String),
    Other(String),
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
            },
            Command::batch(vec![
                Command::perform(load_files(start_path.clone()), Message::FilesLoaded),
                Command::perform(load_directory_structure(start_path), Message::DirectoryStructureLoaded),
                Command::perform(load_system_locations(), Message::SystemLocationsLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        format!("File Explorer - {}", self.current_path.to_string_lossy())
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ChangeDirectory(path) => {
                self.current_path = path.clone();
                self.path_input = path.to_string_lossy().to_string();
                self.selected_file = None;
                self.file_preview = None;
                
                Command::perform(load_files(path), Message::FilesLoaded)
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
                    
                    Command::perform(load_files(path), Message::FilesLoaded)
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
                    // Toggle the expanded state
                    self.directory_structure[index].expanded = !self.directory_structure[index].expanded;
                    
                    // If we're expanding, we might need to reload the directory structure
                    let root_path = self.directory_structure[0].path.clone();
                    Command::perform(load_directory_structure(root_path), Message::DirectoryStructureLoaded)
                } else {
                    Command::none()
                }
            }
            Message::FilePreviewLoaded(preview) => {
                self.file_preview = preview;
                Command::none()
            }
            Message::OpenFile(path) => {
                // Use the open command on various platforms
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
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // Navigation bar at the top
        let navigation_bar = row![
            text_input("Enter path...", &self.path_input)
                .on_input(Message::PathInputChanged)
                .on_submit(Message::NavigateToPath),
            button(text("Go"))
                .on_press(Message::NavigateToPath)
                .padding(5)
        ]
        .spacing(10)
        .padding(10);
        
        // Directory tree on the left
        let directory_tree = self.view_directory_structure();
        
        // File list in the center
        let file_list = self.view_file_list();
        
        // File preview/details on the right
        let file_details = self.view_file_details();
        
        // Main layout
        let content = row![
            container(directory_tree).width(Length::FillPortion(1)),
            container(file_list).width(Length::FillPortion(2)),
            container(file_details).width(Length::FillPortion(1))
        ]
        .spacing(10);
        
        // Combining navigation and content
        column![
            navigation_bar,
            content.height(Length::Fill)
        ]
        .into()
    }
}

impl FileExplorer {
    fn view_directory_structure(&self) -> Element<Message> {
        let tree_title = text("Quick Access")
            .size(16)
            .horizontal_alignment(Horizontal::Center);
        
        let mut dir_list = vec![tree_title.into()];
        
        // First, show system locations (disks and important user folders)
        for location in &self.system_locations {
            let icon = match location.location_type {
                SystemLocationType::Disk => "💾 ",
                SystemLocationType::UserFolder => "📁 ",
            };
            
            let content = row![
                text(format!("{}{}", icon, location.name))
            ]
            .spacing(5);
            
            // Use a darker blue for folders (default style)
            let btn = button(content)
                .on_press(Message::ChangeDirectory(location.path.clone()))
                .width(Length::Fill);
            
            dir_list.push(
                container(btn)
                    .padding(5)
                    .width(Length::Fill)
                    .into()
            );
        }
        
        // Add a separator between quick access and directory structure
        dir_list.push(
            container(
                text("Current Directory")
                    .size(16)
                    .horizontal_alignment(Horizontal::Center)
            )
            .padding([20, 0, 5, 0])
            .into()
        );
        
        // Display the directory structure as an indented list
        for (index, dir) in self.directory_structure.iter().enumerate() {
            // Only show directories that should be visible
            let should_show = true; // Logic can be added here to hide children of collapsed dirs
            
            if should_show {
                // Create indentation based on depth
                let indent = dir.depth * 20; // 20 pixels per level
                
                // Create expand/collapse button if it's a directory
                let toggle_icon = if dir.expanded { "▼ " } else { "▶ " };
                
                let content = row![
                    container(text(toggle_icon))
                        .width(Length::Fixed(20.0)),
                    text(format!("📁 {}", dir.name))
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
                        .into()
                );
            }
        }
        
        scrollable(
            container(
                column(dir_list)
                    .spacing(2)
            )
            .padding(10)
            .height(Length::Fill)
        )
        .into()
    }
    
    fn view_file_list(&self) -> Element<Message> {
        let mut file_list = vec![];
        
        // Title
        file_list.push(
            row![
                text("Name").width(Length::FillPortion(3)),
                text("Size").width(Length::FillPortion(1)),
                text("Modified").width(Length::FillPortion(2))
            ]
            .spacing(10)
            .padding(5)
            .into()
        );
        
        // Parent directory button
        if let Some(parent) = self.current_path.parent() {
            file_list.push(
                button(
                    row![
                        text("📁 .. (Parent Directory)").width(Length::FillPortion(3)),
                        text("").width(Length::FillPortion(1)),
                        text("").width(Length::FillPortion(2))
                    ]
                    .spacing(10)
                )
                .on_press(Message::ChangeDirectory(parent.to_path_buf()))
                .width(Length::Fill)
                .into(),
            );
        }
        
        // Files and directories
        for entry in &self.files {
            let file_name = entry.path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                .to_string_lossy()
                .to_string();
            
            let size_text = format_size(entry.size);
            
            let modified_text = entry.modified
                .map(|time| format_time(time))
                .unwrap_or_else(|| String::from("-"));
            
            let is_selected = self.selected_file.as_ref().map_or(false, |p| p == &entry.path);
            
            let row_content = row![
                text(format!("{} {}", if entry.path.is_dir() { "📁" } else { "📄" }, file_name))
                    .width(Length::FillPortion(3)),
                text(size_text).width(Length::FillPortion(1)),
                text(modified_text).width(Length::FillPortion(2))
            ]
            .spacing(10);
            
            let btn = if entry.path.is_dir() {
                button(row_content)
                    .on_press(Message::ChangeDirectory(entry.path.clone()))
            } else {
                // Use a lighter blue for files
                button(row_content)
                    .on_press(Message::FileSelected(entry.path.clone()))
                    .style(iced::theme::Button::Custom(Box::new(LighterBlueButton)))
            };
            
            let styled_btn = if is_selected {
                btn.style(iced::theme::Button::Positive)
            } else {
                btn
            };
            
            file_list.push(styled_btn.width(Length::Fill).into());
        }
        
        scrollable(
            container(
                column(file_list)
                    .spacing(5)
            )
            .padding(10)
            .height(Length::Fill)
        )
        .into()
    }
    
    fn view_file_details(&self) -> Element<Message> {
        let mut details = vec![];
        
        details.push(
            text("File Details")
                .size(16)
                .horizontal_alignment(Horizontal::Center)
                .into()
        );
        
        if let Some(selected_path) = &self.selected_file {
            let file_name = selected_path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                .to_string_lossy();
            
            let extension = selected_path.extension()
                .unwrap_or_else(|| std::ffi::OsStr::new(""))
                .to_string_lossy()
                .to_string();
            
            // Basic details
            details.push(text(format!("Name: {}", file_name)).into());
            details.push(text(format!("Type: {}", extension.to_uppercase())).into());
            
            // Size
            if let Ok(metadata) = fs::metadata(selected_path) {
                details.push(text(format!("Size: {}", format_size(metadata.len()))).into());
                
                // Modified time
                if let Ok(time) = metadata.modified() {
                    details.push(text(format!("Modified: {}", format_time(time))).into());
                }
            }
            
            details.push(
                button(text("Open File"))
                    .on_press(Message::OpenFile(selected_path.clone()))
                    .padding(5)
                    .into()
            );
            
            // Preview section
            details.push(text("Preview:").size(16).into());
            
            if let Some(preview) = &self.file_preview {
                match preview {
                    FilePreview::Image(handle) => {
                        details.push(
                            container(
                                image(handle.clone())
                                    .width(Length::Fill)
                            )
                            .height(Length::Fixed(200.0))
                            .width(Length::Fill)
                            .center_x()
                            .center_y()
                            .into()
                        );
                    }
                    FilePreview::Text(content) => {
                        let preview_text = if content.len() > 500 {
                            format!("{}...", &content[..500])
                        } else {
                            content.clone()
                        };
                        
                        details.push(
                            scrollable(
                                text(preview_text)
                                    .width(Length::Fill)
                            )
                            .height(Length::Fixed(200.0))
                            .into()
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
            container(
                column(details)
                    .spacing(10)
            )
            .padding(10)
            .height(Length::Fill)
        )
        .into()
    }
}

// Custom button style for files (lighter blue)
struct LighterBlueButton;

impl iced::widget::button::StyleSheet for LighterBlueButton {
    type Style = Theme;
    
    fn active(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = style.active(&iced::theme::Button::Secondary);
        appearance.background = Some(iced::Background::Color(Color::from_rgb(0.6, 0.8, 0.95)));
        appearance
    }
    
    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = style.hovered(&iced::theme::Button::Secondary);
        appearance.background = Some(iced::Background::Color(Color::from_rgb(0.7, 0.85, 0.98)));
        appearance
    }
    
    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut appearance = style.pressed(&iced::theme::Button::Secondary);
        appearance.background = Some(iced::Background::Color(Color::from_rgb(0.5, 0.75, 0.92)));
        appearance
    }
}

// Helper functions
async fn load_files(path: PathBuf) -> Vec<FileEntry> {
    fs::read_dir(&path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| {
                    let path = entry.path();
                    let metadata = fs::metadata(&path).ok();
                    
                    FileEntry {
                        path,
                        size: metadata.as_ref().map_or(0, |m| m.len()),
                        modified: metadata.and_then(|m| m.modified().ok()),
                    }
                })
                .collect()
        })
        .unwrap_or_else(|_| vec![])
}

async fn load_directory_structure(root: PathBuf) -> Vec<DirectoryEntry> {
    let mut dirs = Vec::new();
    
    // Add the root directory
    let root_name = root.file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("Root"))
        .to_string_lossy()
        .to_string();
    
    dirs.push(DirectoryEntry {
        path: root.clone(),
        name: root_name,
        depth: 0,
        expanded: true,
    });
    
    // Load all subdirectories up to a certain depth
    scan_directory(&root, &mut dirs, 0, 3); // Max depth of 3
    
    dirs
}

async fn load_system_locations() -> Vec<SystemLocation> {
    let mut locations = Vec::new();
    
    // Add system drives (different for each OS)
    #[cfg(target_os = "windows")]
    {
        // Windows drives
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
        // macOS volumes
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
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
        // Linux mounts
        if let Ok(entries) = fs::read_dir("/media") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
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
    
    // Add important user folders
    if let Some(home_dir) = dirs_next::home_dir() {
        locations.push(SystemLocation {
            name: "Home".to_string(),
            path: home_dir.clone(),
            location_type: SystemLocationType::UserFolder,
        });
        
        // Add common user folders
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

fn scan_directory(path: &Path, dirs: &mut Vec<DirectoryEntry>, depth: usize, max_depth: usize) {
    if depth >= max_depth {
        return;
    }
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                let name = entry_path.file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("Unknown"))
                    .to_string_lossy()
                    .to_string();
                
                dirs.push(DirectoryEntry {
                    path: entry_path.clone(),
                    name,
                    depth: depth + 1,
                    expanded: false,
                });
                
                // Recursively scan subdirectories
                scan_directory(&entry_path, dirs, depth + 1, max_depth);
            }
        }
    }
}

async fn load_file_preview(path: PathBuf) -> Option<FilePreview> {
    if !path.is_file() {
        return None;
    }
    
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match extension.as_str() {
        // Image files
        "jpg" | "jpeg" | "png" | "gif" | "bmp" => {
            if let Ok(bytes) = fs::read(&path) {
                return Some(FilePreview::Image(image::Handle::from_memory(bytes)));
            }
        }
        
        // Text files
        "txt" | "md" | "rs" | "toml" | "json" | "yaml" | "yml" | "css" | "html" | "js" | "py" => {
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(FilePreview::Text(content));
            }
        }
        
        // PDF files
        "pdf" => {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            return Some(FilePreview::Pdf(format!("PDF document ({} bytes)\nUse 'Open File' to view", size)));
        }
        
        // Other file types
        _ => {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            return Some(FilePreview::Other(format!("{} file ({} bytes)\nUse 'Open File' to open", extension.to_uppercase(), size)));
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
    
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    let secs = duration.as_secs();
    
    // Basic formatting - in a real app you'd use chrono crate
    let seconds = secs % 60;
    let minutes = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    let days = (secs / 86400) % 30;
    let months = (secs / 2592000) % 12;
    let years = secs / 31104000;
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
        1970 + years, 1 + months, 1 + days, hours, minutes, seconds)
}