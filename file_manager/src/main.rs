use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

fn setup_database(conn: &Connection) {
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS files USING fts5(name, path);",
        [],
    )
    .unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_meta (path TEXT PRIMARY KEY, modified INTEGER);",
        [],
    )
    .unwrap();
}

fn upsert_file(conn: &Connection, path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        let modified = metadata
            .modified()
            .ok()
            .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let path_str = path.to_string_lossy().to_string();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let exists: Option<i64> = conn
            .query_row(
                "SELECT modified FROM file_meta WHERE path = ?1",
                params![path_str],
                |row| row.get(0),
            )
            .optional()
            .unwrap();

        if exists != Some(modified) {
            conn.execute(
                "INSERT OR REPLACE INTO files (rowid, name, path) VALUES ((SELECT rowid FROM files WHERE path = ?1), ?2, ?1);",
                params![path_str, name]
            ).unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO file_meta (path, modified) VALUES (?1, ?2);",
                params![path_str, modified],
            )
            .unwrap();
        }
    }
}

fn remove_file(conn: &Connection, path: &Path) {
    let path_str = path.to_string_lossy();
    conn.execute("DELETE FROM files WHERE path = ?1;", params![path_str])
        .unwrap();
    conn.execute("DELETE FROM file_meta WHERE path = ?1;", params![path_str])
        .unwrap();
}

fn initial_scan(conn: &Connection, root: &Path) {
    let ignored = ["/proc", "/sys", "/dev"];
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| !ignored.iter().any(|ignore| e.path().starts_with(ignore)))
    {
        upsert_file(conn, &entry.path());
    }
}

fn update_from_journald(conn: &Connection, root: &Path) {
    let journalctl = Command::new("journalctl")
        .arg("_COMM=inotify")
        .arg("--since=yesterday")
        .output()
        .expect("failed to run journalctl");

    let output = String::from_utf8_lossy(&journalctl.stdout);
    for line in output.lines() {
        if line.contains("CREATE") || line.contains("MODIFY") {
            if let Some(start) = line.find("/") {
                let path = PathBuf::from(&line[start..]);
                if path.starts_with(root) && path.exists() {
                    upsert_file(conn, &path);
                }
            }
        } else if line.contains("DELETE") {
            if let Some(start) = line.find("/") {
                let path = PathBuf::from(&line[start..]);
                if path.starts_with(root) {
                    remove_file(conn, &path);
                }
            }
        }
    }
}

fn watch_directory(conn: Connection, path: PathBuf) {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default()).unwrap();
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    for res in rx {
        if let Ok(event) = res {
            for path in event.paths {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        if path.is_file() {
                            upsert_file(&conn, &path);
                        }
                    }
                    EventKind::Remove(_) => {
                        remove_file(&conn, &path);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn search(conn: &Connection, query: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Exact match with FTS
    let mut stmt = conn
        .prepare("SELECT path FROM files WHERE name MATCH ?1;")
        .unwrap();
    let rows = stmt
        .query_map([query], |row| row.get::<_, String>(0))
        .unwrap();
    for row in rows.flatten() {
        results.push(row);
    }

    if !results.is_empty() {
        return results;
    }

    // Regex fallback
    let re = Regex::new(query).unwrap_or(Regex::new(".*").unwrap());
    let mut stmt = conn.prepare("SELECT name, path FROM files;").unwrap();
    let rows = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let path: String = row.get(1)?;
            Ok((name, path))
        })
        .unwrap();

    for row in rows.flatten() {
        if re.is_match(&row.0) {
            results.push(row.1);
        }
    }

    if !results.is_empty() {
        return results;
    }

    // Fuzzy fallback
    for row in stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let path: String = row.get(1)?;
            Ok((name, path))
        })
        .unwrap()
        .flatten()
    {
        if levenshtein::levenshtein(&row.0, query) <= 2 {
            results.push(row.1);
        }
    }

    results
}

fn main() {
    let root = PathBuf::from("/");
    let conn = Connection::open("files.db").unwrap();
    setup_database(&conn);
    //    update_from_journald(&conn, &root);
    initial_scan(&conn, &root);

    std::thread::spawn(move || watch_directory(conn, root));

    let conn2 = Connection::open("files.db").unwrap();
    let results = search(&conn2, "main.rs");
    for path in results {
        println!("{}", path);
    }
}
