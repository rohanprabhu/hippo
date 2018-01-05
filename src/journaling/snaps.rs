use journaling::journal::*;
use std::path::PathBuf;
use std::string::ToString;

pub struct Snaps<'a> {
    journal: &'a Journal
}

impl <'a> Snaps<'a> {
    pub fn for_journal(journal: &Journal) -> Snaps {
        Snaps {
            journal
        }
    }

    pub fn create_snapshot(&self, file_path: &PathBuf) -> SnapshotEntry {
        if self.journal.contains_record(file_path.into_os_string().into_string().unwrap()) {

        }
    }

    fn new_managed_file(&self) {

    }
}
