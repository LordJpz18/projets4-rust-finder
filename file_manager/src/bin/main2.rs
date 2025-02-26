use std::path::Path;
use std::time::Instant;
use file_manager::files;
use file_manager::files::display_files::print_from_dir;
use file_manager::research;
use research::{
    find::find_file,
    find::build_file_tree,
    find::find_file_btree
};


fn main() 
{
    // Utiliser le dossier personnel comme point de départ (équivalent à ~/)
    let path = std::env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
    //let dossiers = ["Desktop", "Documents", "Downloads", "Movies", "Music", "Pictures"];
    //for dossier in dossiers {
        let chemin = Path::new(&path).join("Documents"); 
        match chemin.to_str() {
            Some(chemin_str) => {
                //println!("Contenu de {} :", dossier); // Optional: header for clarity
                if let Err(e) = print_from_dir(chemin_str) {
                    eprintln!("Error in dossier: {}", e);
                }
                println!();
            }
            None => eprintln!("Invalid path for dossiers (non-UTF-8)"),
        }       
    }
