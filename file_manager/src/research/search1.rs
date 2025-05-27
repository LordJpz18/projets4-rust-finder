use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Fonctions inchangées
pub fn has_execute_permission(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn is_binary_executable(path: &Path) -> bool {
    let mut header = [0; 4];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_ok()
        && (header == [0x7F, b'E', b'L', b'F']
            || header == [0xFE, 0xED, 0xFA, 0xCE]
            || header == [0xFE, 0xED, 0xFA, 0xCF])
}

pub fn has_shebang(path: &Path) -> bool {
    let mut shebang = [0; 2];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut shebang))
        .is_ok()
        && shebang == [b'#', b'!']
}

pub fn is_application(path: &Path) -> bool {
    path.is_file()
        && (has_execute_permission(path) || is_binary_executable(path) || has_shebang(path))
}

pub fn get_modified_time(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|meta| meta.mtime() as u64)
}

// Dossiers pertinents pour utilisateurs non développeurs
fn get_relevant_dirs() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    vec![
        dirs::document_dir().unwrap_or_else(|| home.join("Documents")),
        dirs::download_dir().unwrap_or_else(|| home.join("Downloads")),
        dirs::desktop_dir().unwrap_or_else(|| home.join("Desktop")),
        dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")),
        dirs::video_dir().unwrap_or_else(|| home.join("Videos")),
    ]
}

pub fn find_recent_files() -> Vec<(PathBuf, u64)> {
    let mut heap: BinaryHeap<Reverse<(u64, PathBuf)>> = BinaryHeap::new();
    let relevant_dirs = get_relevant_dirs();

    let paths: Vec<PathBuf> = relevant_dirs
        .into_par_iter()
        .flat_map(|dir| {
            WalkDir::new(dir)
                .into_iter()
                .filter_map(Result::ok)
                .map(|entry| entry.path().to_path_buf())
                .collect::<Vec<PathBuf>>()
        })
        .collect();

    for path in paths {
        if let Some(modified) = get_modified_time(&path) {
            heap.push(Reverse((modified, path)));
            if heap.len() > 75 {
                heap.pop();
            }
        }
    }

    heap.into_sorted_vec()
        .into_iter()
        .map(|Reverse((modified, path))| (path, modified))
        .collect()
}

// Charge les chemins dans une Vec<String> à partir des dossiers pertinents
pub fn build_file_tree() -> Vec<String> {
    let relevant_dirs = get_relevant_dirs();
    relevant_dirs
        .into_iter()
        .flat_map(|dir| {
            WalkDir::new(dir)
                .max_depth(5) // Profondeur raisonnable pour inclure les sous-dossiers
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|entry| entry.path().to_string_lossy().into_owned())
                .collect::<Vec<String>>()
        })
        .collect()
}

// Recherche dans une Vec<String>
pub fn find_file_btree(query: &str, tree: &[String]) -> Vec<String> {
    let query_lower = query.to_lowercase();
    tree.iter()
        .filter(|path| path.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
