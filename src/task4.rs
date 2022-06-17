use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use walkdir::WalkDir;

pub fn search_files(dir: impl AsRef<Path>, ext: &str) -> Result<(), anyhow::Error> {
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry
            .file_name()
            .to_string_lossy()
            .ends_with(&[".", ext].concat())
        {
            let filepath = entry.path();
            let file = File::open(filepath)?;
            let reader = BufReader::new(file);
            let mut nlines = 0;
            for _ in reader.lines() {
                nlines += 1;
            }
            println!("{} {}", filepath.to_string_lossy(), nlines);
        }
    }
    Ok(())
}
