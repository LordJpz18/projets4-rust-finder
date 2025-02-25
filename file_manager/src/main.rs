use std::path::Path;

mod files;
use files::display_files::print_from_dir;

use file_manager::files::my_functions::*;
use std::io;

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
    let mut permissions_vec = Vec::new();

    loop
    {
        println!("\n1) Create file (predefined extensions)");
        println!("2) Create file");
        println!("3) Create directory");
        println!("4) Delete a file");
        println!("5) Print file permissions");
        println!("6) Quit");

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice
        {
            "1" => create_file_with_ext(&mut permissions_vec),
            "2" => create_file(&mut permissions_vec),
            "3" => create_directory(),
            "4" => delete_file(),
            "5" =>
            {
                println!("\nEnter the file name to show permissions:");
                let mut file_name = String::new();
                io::stdin().read_line(&mut file_name).unwrap();
                let file_name = file_name.trim();

                let f_found = permissions_vec.iter().find(|f| f.name == file_name);

                if f_found.is_some()
                {
                    f_found.unwrap().print_perms();
                }
                else
                {
                    println!("File '{}' not found!", file_name);
                }
            }
            "6" =>
            {
                println!("Quitting...");
                break;
            }

            _ => println!("Put a valid number"),
        }
    }
}
