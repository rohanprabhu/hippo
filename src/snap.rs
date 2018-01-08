use std::path::PathBuf;
use journaling::journal::Journal;
use journaling::managed_file_journal::ManagedFileJournal;

pub fn snap(journal: &mut Journal, file_paths: Vec<PathBuf>) {
    let mut managed_file_journal = ManagedFileJournal::for_journal(journal);

    for file_path in file_paths {
        println!("Creating snapshot for {:?}", file_path);
        let mut managed_file = managed_file_journal.get_managed_file(&file_path);
        managed_file.snap_current_state(None, None, None);
    }
}
