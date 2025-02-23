use std::path::Path;

mod files;
use files::display_files::print_from_dir;


/*
fn main() {
    // Example : go from root ("/" sur Unix, "C:\" sur Windows)
    let root = if cfg!(target_os = "windows") {
        "C:\\"
    } else {
        "/"
    };

    match print_dir(root) {
        Ok(()) => println!("End of traversal."),
        Err(e) => eprintln!("Errors encountered. : {}", e),
    }
}
*/
fn main() {
    // Utiliser le dossier personnel comme point de départ (équivalent à ~/)
    let path = std::env::var("HOME").unwrap_or_else(|_| "/Users/user".to_string());
    let dossiers = ["Desktop", "Documents", "Downloads", "Movies", "Music", "Pictures"];
    for dossier in dossiers {
        let chemin = Path::new(&path).join(dossier); 
        match chemin.to_str() {
            Some(chemin_str) => {
                //println!("Contenu de {} :", dossier); // Optional: header for clarity
                if let Err(e) = print_from_dir(chemin_str) {
                    eprintln!("Error in {}: {}", dossier, e);
                }
                println!(); // Blank line between folders
            }
            None => eprintln!("Invalid path for {} (non-UTF-8)", dossier),
        }       
    }
    //println!("Dossiers pertinents dans {} :", path);
    //if let Err(e) = print_dir(&path) {
     //   eprintln!("Erreur : {}", e);
    //}

    /* Optionnel : lister les volumes montés
    println!("\nVolumes montés :");
    if let Err(e) = print_dir("/Volumes") {
        eprintln!("Erreur : {}", e);
    }
    */
}
