use std::path::Path;
use std::time::Instant;
use file_manager::files;
use files::display_files::print_from_dir;
use file_manager::research;
use research::{
    find::find_file,
    find::build_file_tree,
    find::find_file_btree
};

fn main() {
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












































