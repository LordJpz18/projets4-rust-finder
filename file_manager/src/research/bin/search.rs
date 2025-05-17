use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Vérifie si un fichier est exécutable (permissions Unix)
pub fn has_execute_permission(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

// Vérifie si un fichier est un binaire ELF (Linux) ou Mach-O (macOS)
pub fn is_binary_executable(path: &Path) -> bool {
    let mut header = [0; 4];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .is_ok()
        && (header == [0x7F, b'E', b'L', b'F']
            || header == [0xFE, 0xED, 0xFA, 0xCE]
            || header == [0xFE, 0xED, 0xFA, 0xCF])
}

// Vérifie si un fichier est un script avec un shebang (#!)
pub fn has_shebang(path: &Path) -> bool {
    let mut shebang = [0; 2];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut shebang))
        .is_ok()
        && shebang == [b'#', b'!']
}

// Vérifie si un fichier est une application sous Unix
pub fn is_application(path: &Path) -> bool {
    path.is_file()
        && (has_execute_permission(path) || is_binary_executable(path) || has_shebang(path))
}

// Récupère le timestamp de la dernière modification d'un fichier
pub fn get_modified_time(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|meta| meta.mtime() as u64)
}

// Récupère les dossiers pertinents (Documents, Téléchargements, Bureau)
fn get_relevant_dirs() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    vec![
        dirs::document_dir().unwrap_or_else(|| home.join("Documents")),
        dirs::download_dir().unwrap_or_else(|| home.join("Downloads")),
        dirs::desktop_dir().unwrap_or_else(|| home.join("Desktop")),
    ]
}

// Recherche les fichiers et dossiers récents dans les dossiers pertinents
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

// Charge le disque en tant que BTreeMap avec fichiers et dossiers
pub fn build_file_tree() -> BTreeMap<String, String> {
    let mut tree = BTreeMap::new();
    let root = if cfg!(target_os = "windows") { "C:\\" } else { "/" };
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path_str = entry.path().to_string_lossy().into_owned();
        if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
            tree.insert(name.to_string(), path_str);
        }
    }
    tree
}

// Recherche des fichiers et dossiers contenant la séquence dans leur chemin
pub fn find_file_btree(query: &str, tree: &BTreeMap<String, String>) -> Vec<String> {
    let query_lower = query.to_lowercase();
    tree.values()
        .filter(|path| path.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
