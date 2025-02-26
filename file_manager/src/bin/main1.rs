use file_manager::files::my_functions::*;
use std::io;

fn main() 
{
    let mut permissions_vec = Vec::new();

    loop 
    {
        println!("\n1) Create file (predefined extensions)");
        println!("2) Create file");
        println!("3) Create directory");
        println!("4) Delete a file");
        println!("5) Print file permissions");
        println!("6) Update file permissions");
        println!("7) Quit");

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
                println!("\nEnter the file name to update permissions:");
                let mut file_name = String::new();
                io::stdin().read_line(&mut file_name).unwrap();
                let file_name = file_name.trim();

                let f_found = permissions_vec.iter_mut().find(|f| f.name == file_name);

                if let Some(fichier) = f_found 
                {

                    println!("Read permission (y/n):");
                    let mut read = String::new();
                    io::stdin().read_line(&mut read).unwrap();
                    let read = read.trim().to_lowercase() == "y";

                    println!("Write permission (y/n):");
                    let mut write = String::new();
                    io::stdin().read_line(&mut write).unwrap();
                    let write = write.trim().to_lowercase() == "y";

                    println!("Delete permission (y/n):");
                    let mut delete = String::new();
                    io::stdin().read_line(&mut delete).unwrap();
                    let delete = delete.trim().to_lowercase() == "y";

                    fichier.update_perms(read, write, delete);
                    println!("Permissions updated for file '{}'.", file_name);
                }
                else 
                {
                    println!("File '{}' not found!", file_name);
                }
            }
            "7" => 
            {
                println!("Quitting...");
                break;
            }
             
            _ => println!("Put a valid number"),
        }
    }
}


