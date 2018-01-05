use std::path::PathBuf;
use journaling::journal::Journal;
use journaling::snaps::Snaps;

pub fn snap(journal: &Journal, file_paths: Vec<PathBuf>) {
    println!("Creating snapshot for {:?}", file_paths);

    let snaps = Snaps::for_journal(journal);
}
