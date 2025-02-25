use walkdir::WalkDir;
//use std::path::Path;
use std::collections::BTreeMap;

// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.to_ascii_lowercase().chars().enumerate() {
        for (j, c2) in s2.to_ascii_lowercase().chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i + 1][j] + 1)
                .min(matrix[i][j + 1] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

// Calculate similarity percentage based on Levenshtein distance
fn similarity(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2) as f64;
    let max_len = s1.chars().count().max(s2.chars().count()) as f64;
    if max_len == 0.0 {
        1.0
    } else {
        1.0 - (distance / max_len)
    }
}

// Search for a file from the root and return matches or similar files
pub fn find_file(filename: &str) -> Vec<String> {
    let root = if cfg!(target_os = "windows") {
        "C:\\"
    } else {
        "/"
    };
    let mut exact_match = None;
    let mut similar_files = Vec::new();

    // Traverse the filesystem from the root
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok())
    // Skip errors (e.g., permission denied)
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Check for exact match (case-insensitive)
                if file_name.eq_ignore_ascii_case(filename) {
                    exact_match = Some(path.to_string_lossy().into_owned());
                    break; // Stop searching once exact match is found
                }
                // Calculate similarity
                let sim = similarity(file_name, filename);
                if sim > 0.8 {
                    similar_files.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    // Return results
    if let Some(exact) = exact_match {
        vec![exact] // Return only the exact match if found
    } else {
        similar_files // Return similar files if no exact match
    }
}

//Btree reprentation of the disk
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

//search function with BTree and levenshtein
pub fn find_file_btree(filename: &str, tree: &BTreeMap<String, String>) -> Vec<String> {
    if let Some(path) = tree.get(filename) {
        return vec![path.clone()];
    }
    tree.iter()
        .filter(|(name, _)| similarity(name, filename) > 0.8)
        .map(|(_, path)| path.clone())
        .collect()
}

/*

fn main() {
    let search_file = "readme.txt";
    println!("Searching for '{}'", search_file);
    let start = Instant::now();
    let results = find_file(search_file);
    let duration = start.elapsed();

    let B = build_file_tree();
    let s2 = Instant::now();
    let res = find_file_btree(search_file, B);
    let d2 = start.elapsed()


    println!("Execution time: {} ms", duration.as_millis());
    if results.is_empty() {
        println!("No similar files found.");
    } else if results.len() == 1 && results[0].ends_with(search_file) {
        println!("Exact match found: {}", results[0]);
    } else {
        println!("Similar files found (>80% similarity):");
        for path in &results {
            println!("- {}", path);
        }
    }
}*/
