use std::path::PathBuf;
use journaling::journal::Journal;
use journaling::managed_file_journal::{ManagedFileJournal, ManagedFileJournalError};
use journaling::managed_file::ManagedFileError;

pub struct SnapError;

impl From<ManagedFileError> for SnapError {
    fn from(_: ManagedFileError) -> Self { SnapError }
}

impl From<ManagedFileJournalError> for SnapError {
    fn from(_: ManagedFileJournalError) -> Self { SnapError }
}

pub fn snap(
    journal: &mut Journal, name: Option<String>, comment: Option<String>, file_paths: Vec<PathBuf>
) -> Result<(), SnapError> {
    let mut managed_file_journal = ManagedFileJournal::for_journal(journal);

    for file_path in file_paths {
        println!("Creating snapshot for {}", file_path.to_owned().into_os_string().into_string().unwrap());

        let managed_file = managed_file_journal.create_or_get_managed_file(&file_path);
        managed_file?.snap_current_state(name.to_owned(), comment.to_owned(), None)?;
    }

    Ok(())
}
