use std::path::Path;
use std::time::Instant;
mod files;
use files::display_files::print_from_dir;
mod research;
use research::{
    find::find_file,
    find::build_file_tree,
    find::find_file_btree
};
use file_manager::files::my_functions::*;
use std::io;


//Clémence's Tests
//com
fn main() {
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











//My Test (JP) 1st
fn main2() {
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



//Tets search
fn main3() {
    let search_file = "attestation.pdf";
    println!("Searching for '{}'", search_file);
    let start = Instant::now();
    let results = find_file(search_file);
    let duration = start.elapsed();
    
    println!("Linear search time: {} ms", duration.as_millis());
    
    let bt = build_file_tree();
    let s2 = Instant::now();
    let res = find_file_btree(search_file, &bt);
    let d2 = s2.elapsed();

    println!("BTree search time: {} ms", d2.as_millis());

    if res.is_empty() {
        println!("No similar files found.");
    } else if res.len() == 1 && res[0].ends_with(search_file) {
        println!("Exact match found: {}", res[0]);
    } else {
        println!("Similar files found (>80% similarity):");
        for path in &res {
            println!("- {}", path);
        }
    }
}




//end com











































