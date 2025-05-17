use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
use std::fs;
use std::io::Read;
//#[cfg(target_os = "unix")]
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
//use std::time::SystemTime;
use walkdir::WalkDir;

// Récupère le timestamp de la dernière modification d'un fichier
pub fn get_modified_time(path: &Path) -> Option<u64> {
    if let Ok(metadata) = fs::metadata(path) {
        //    #[cfg(target_os = "windows")]
        {
            return Some(metadata.last_write_time() as u64);
        }
        //      #[cfg(target_os = "unix")]
        {
            return Some(metadata.mtime() as u64);
        }
    }
    None
}

// Récupère le dossier utilisateur
pub fn get_user_home() -> PathBuf {
    dirs::home_dir().expect("Impossible de récupérer le dossier utilisateur")
}

// Parcours récursif du disque, collecte les 50 fichiers les plus récents
pub fn find_recent_files(root: &Path) -> Vec<(PathBuf, u64)> {
    let mut heap: BinaryHeap<Reverse<(u64, PathBuf)>> = BinaryHeap::new();

    let paths: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .par_bridge() // Parcours parallèle
        .filter_map(Result::ok) // Ignore les erreurs
        .filter(|entry| entry.file_type().is_file()) // Garde uniquement les fichiers
        .map(|entry| entry.path().to_path_buf())
        .collect();

    for path in paths {
        if let Some(modified) = get_modified_time(&path) {
            heap.push(Reverse((modified, path)));

            if heap.len() > 50 {
                heap.pop(); // Conserve uniquement les 50 plus récents
            }
        }
    }

    heap.into_sorted_vec()
        .into_iter()
        .map(|Reverse(x)| x)
        .collect()
}

//Load the disk as a Btree Map
pub fn build_file_tree() -> BTreeMap<String, String> {
    let mut tree = BTreeMap::new();
    let root = if cfg!(target_os = "windows") {
        "C:\\"
    } else {
        "/"
    };
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                tree.insert(
                    name.to_string(),
                    entry.path().to_string_lossy().into_owned(),
                );
            }
        }
    }
    tree
}

//Search a file by its name
pub fn find_file_btree(filename: &str, tree: &BTreeMap<String, String>) -> Vec<String> {
    // if the file is found it's directely returned
    if let Some(path) = tree.get(filename) {
        return vec![path.clone()];
    }

    // return a vector of files with similar name
    let prefix = filename;
    tree.range(prefix..)
        .take_while(|(key, _)| key.starts_with(prefix))
        .map(|(_, path)| path.clone())
        .collect()
}

// Vérifie si un fichier est une application (Windows, Linux, macOS)
pub fn is_application(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    // Vérifier les extensions Windows
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        if ["exe", "bat", "cmd"].contains(&ext_str.as_str()) {
            return true;
        }
    }

    // Vérifier les permissions d'exécution (Linux/macOS)
    #[cfg(unix)]
    {
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 != 0 {
                return true;
            }
        }
    }

    // Lire le header du fichier pour identifier les binaires
    if let Ok(mut file) = fs::File::open(path) {
        let mut header = [0; 4];
        if file.read_exact(&mut header).is_ok() {
            return header == [b'M', b'Z'] // Windows (PE)
                || header == [0x7F, b'E', b'L', b'F'] // Linux (ELF)
                || header == [0xFE, 0xED, 0xFA, 0xCE] // macOS (Mach-O 32-bit)
                || header == [0xFE, 0xED, 0xFA, 0xCF]; // macOS (Mach-O 64-bit)
        }
    }

    // Vérifier si c'est un script avec un shebang (#!)
    if let Ok(mut file) = fs::File::open(path) {
        let mut shebang = [0; 2];
        if file.read_exact(&mut shebang).is_ok() && shebang == [b'#', b'!'] {
            return true;
        }
    }

    false
}
