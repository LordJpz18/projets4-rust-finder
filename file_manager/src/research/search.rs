use levenshtein::levenshtein;
use notify::{
    Config, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use regex::Regex;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use walkdir::WalkDir;

pub fn initialize_db(conn: &Connection) {
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS files USING fts5(name, path);",
        [],
    )
    .unwrap();
}

pub fn insert_file(conn: &Connection, path: &Path) {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let path_str = path.to_string_lossy();
        conn.execute(
            "INSERT INTO files (name, path) VALUES (?1, ?2);",
            params![name, path_str],
        )
        .ok();
    }
}

pub fn remove_file(conn: &Connection, path: &Path) {
    let path_str = path.to_string_lossy();
    conn.execute("DELETE FROM files WHERE path = ?1;", params![path_str])
        .ok();
}

pub fn initial_scan(conn: &Connection, root: &Path) {
    let ignored_dirs: HashSet<&str> = [
        "/proc",
        "/sys",
        "/dev",
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\programdata",
        "c:\\$recycle.bin",
        "c:\\system volume information",
        "c:\\recovery",
        "c:\\perflogs",
    ]
    .into_iter()
    .collect();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && !ignored_dirs.iter().any(|d| path.starts_with(d)) {
            insert_file(conn, path);
        }
    }
}

pub fn find(conn: &Connection, query: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut stmt = conn
        .prepare("SELECT path FROM files WHERE name MATCH ?1;")
        .unwrap();
    let mut rows = stmt.query(params![query]).unwrap();

    while let Some(row) = rows.next().unwrap() {
        results.push(row.get::<_, String>(0).unwrap());
    }

    if results.is_empty() {
        let regex = Regex::new(query).unwrap();
        let mut stmt = conn.prepare("SELECT name, path FROM files;").unwrap();
        let mut rows = stmt.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let name: String = row.get(0).unwrap();
            let path: String = row.get(1).unwrap();
            if regex.is_match(&name) {
                results.push(path);
            }
        }
    }

    if results.is_empty() {
        let mut stmt = conn.prepare("SELECT name, path FROM files;").unwrap();
        let mut rows = stmt.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let name: String = row.get(0).unwrap();
            let path: String = row.get(1).unwrap();
            if levenshtein(&name, query) <= 2 {
                results.push(path);
            }
        }
    }

    results
}
