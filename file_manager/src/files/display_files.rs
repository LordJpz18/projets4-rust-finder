use walkdir::WalkDir;
use colored::Colorize;
//use std::path::Path;
pub fn print_from_dir(path: &str) -> Result<(), walkdir::Error> // Eventually add a bool parameter
                                                                // display or not cached files
                                                                // later
{

    for entry in WalkDir::new(path).max_depth(3).into_iter() {
        match entry {
            Ok(dir_entry) => {
                let name = dir_entry.file_name().to_string_lossy();
                if dir_entry.file_type().is_file() {
                    if name.starts_with('.'){
                        println!("{}", name.truecolor(128, 128, 128));
                    }
                    else{
                        println!("{}", name);
                    }
                }
                else if dir_entry.file_type().is_dir() {
                    println!("{}", name.blue().bold());
                } 
                else {
                    println!("{}", name.yellow());
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

