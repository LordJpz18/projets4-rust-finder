// main.rs

mod research;
use research::{
    search::build_file_tree, search::find_file_btree, search::find_recent_files,
    search::is_application,
};
mod gui;
use gui::gui::FileExplorer;
use iced::Result;
use iced::Sandbox;

fn main() -> Result {
    FileExplorer::run(iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(800.0, 600.0),
            ..Default::default()
        },
        ..Default::default()
    })
}
