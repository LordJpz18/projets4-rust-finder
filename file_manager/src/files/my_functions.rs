use std::fs::{self, File, create_dir};
use std::path::Path;
use std::io;

pub struct PermissionsFichier 
{
    pub name: String,
    pub is_read: bool,
    pub is_write: bool,
    pub is_delete: bool,
}

#[allow(dead_code)]
impl PermissionsFichier 
{
    pub fn new(name: &str) -> Self 
    {
        Self 
        {
            name: name.to_string(),
            is_read: true,
            is_write: true,
            is_delete: false,
        }
    }

    pub fn print_perms(&self) 
    {
        println!("Permissions for {}:", self.name);
        
        if self.is_read 
        {
            println!("Read : Yes");
        } 
        else 
        {
            println!("Read : No");
        }
        if self.is_write
        {
            println!("Write : Yes");
        }
        else
        {
            println!("Write : No");
        }

        if self.is_delete
        {
            println!("Delete : Yes");
        }
        else
        {
            println!("Delete : No");
        }
    }
}


pub fn is_name_valid(name: &str) -> bool 
{
    for c in name.chars() 
    {
        if !c.is_alphanumeric() && c != '.' && c != '_'
        {
            return false;
        }
    }
    return true;
}

pub fn check_existence(path: &str) -> bool 
{
    let path2 = Path::new(path);
    return path2.exists();
}

pub fn create_file_with_ext(permissions_vec: &mut Vec<PermissionsFichier>) 
{
    let extensions = ["png", "jpg", "rs", "c", "asm"];

    //Ask extension
    println!("Choose an extension among:");
    for (i, e) in extensions.iter().enumerate() 
    {
        println!("{} : .{}", i + 1, e);
    }

    println!("Your choice: ");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim(); //removes tabs or spaces

    //verify input is number
    let mut is_number = true;
    for c in choice.chars() 
    {
        if c < '0' || c > '9' 
        {
            is_number = false;
            break;
        }
    }

    if !is_number 
    {
        println!("Invalid choice!");
        return;
    }

    //input string to int
    let mut nb = 0;
    for c in choice.chars()
    {
        nb = nb * 10 + (c as usize - '0' as usize);
    }

    //is input valid
    if nb == 0 || nb > extensions.len() 
    {
        println!("Invalid choice!");
        return;
    }

    //ask name of file
    println!("Enter name of the file (without extension): ");

    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    if !is_name_valid(name) 
    {
        println!("Invalid name!");
        return;
    }

    let path = name.to_string() + "." + &extensions[nb-1];


    if check_existence(&path) 
    {
        println!("File '{}' already exists !", path);
        return;
    }

    //Create file
    let _ = File::create(&path);

    //add perms to the file
    permissions_vec.push(PermissionsFichier::new(&path));

    println!("File '{}' successfully created", path);
}


pub fn create_file(permissions_vec: &mut Vec<PermissionsFichier>) 
{
    println!("Enter file name (without extension)");

    let mut name = String::new();

    io::stdin().read_line(&mut name).unwrap();

    let name = name.trim();

    if !is_name_valid(name) 
    {
        println!("invalid name !");
        return;
    }

    println!("Enter extension ");

    let mut extension = String::new();

    io::stdin().read_line(&mut extension).unwrap();

    let extension = extension.trim();

    if !is_name_valid(extension) 
    {
        println!("invalid extension !");
        return;
    }

    let path = name.to_string() + "." + &extension;


    //ckeck existence
    if check_existence(&path)
    {
        println!("File '{}' already exists!", path);
        return;
    }

    //Create file
    let _ =File::create(&path);

    //add perms to file
    permissions_vec.push(PermissionsFichier::new(&path));
    println!("File '{}' created", path);
}


pub fn create_directory() 
{
    println!("Enter directory name :");

    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    if !is_name_valid(name) 
    {
        println!("invalid name !");
        return;
    }

    if check_existence(name) 
    {
        println!("directory '{}' already exists.", name);
        return;
    }

    let _ =create_dir(name);
}

//Delete file only if it is in bin
pub fn delete_file() 
{
    println!("Which file do you want to delete ?");

    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    let bin_path = format!("bin/{}", name);

    if check_existence(&bin_path) 
    {
        let res = fs::remove_file(&bin_path);
        if let Err(e) = res 
        {
            println!("Error deleting file '{}': {}", name, e);
        }
        else 
        {
            println!("File '{}' deleted.", name);
        }
    } 
    else 
    {
        if check_existence(name) 
        {
            let _ =fs::create_dir_all("bin");
            let _ = fs::rename(name, &bin_path);
            println!("File moved to bin");
        } 
        else 
        {
            println!("File '{}' was not found", name);
        }
    }
}



