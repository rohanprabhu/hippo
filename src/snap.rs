use std::path::PathBuf;
use journaling::journal::Journal;

pub fn snap(_: &Journal, file_paths: Vec<PathBuf>) {
    println!("Creating snapshot for {:?}", file_paths);
}
