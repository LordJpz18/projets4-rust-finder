use rusqlite::Connection;
use std::path::PathBuf;
use std::thread;
mod research;

use research::search::{find, initial_scan, initialize_db};

fn main() {
    let conn = Connection::open("files.db").expect("Failed to open database");
    initialize_db(&conn);

    #[cfg(target_os = "windows")]
    let root = PathBuf::from(r"C:\Users\guill\Documents");
    let root_clone = PathBuf::from(r"C:\Users\guill\Documents");
    #[cfg(not(target_os = "windows"))]
    let root = PathBuf::from("/");
    let root_clone = PathBuf::from("/");

    let conn_clone = Connection::open("files.db").expect("Failed to open database");

    thread::spawn(move || {
        initial_scan(&conn_clone, &root_clone);
    });

    println!("searching ...");

    let query = "enonce.pdf";
    let results = find(&conn, query);
    dbg!(results);
}
