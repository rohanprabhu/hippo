use std::path::PathBuf;
use journaling::journal::Journal;
use journaling::managed_file_journal::ManagedFileJournal;

pub fn snap(journal: &mut Journal, name: Option<String>, comment: Option<String>, file_paths: Vec<PathBuf>) {
    let mut managed_file_journal = ManagedFileJournal::for_journal(journal);

    for file_path in file_paths {
        println!("Creating snapshot for {}", file_path.to_owned().into_os_string().into_string().unwrap());

        let mut managed_file = managed_file_journal.create_or_get_managed_file(&file_path);
        managed_file.snap_current_state(name.to_owned(), comment.to_owned(), None);
    }
}
