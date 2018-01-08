extern crate term;
extern crate uuid;

use std::path::PathBuf;
use std::path::Path;
use std::fs::{create_dir_all, OpenOptions};

use journaling::journal::*;
use journaling::managed_file::*;

use self::uuid::Uuid;

static MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME: &'static str = "__snaps_journal";

pub struct ManagedFileJournal<'a> {
    root_journal: &'a mut Journal
}

impl <'a> ManagedFileJournal<'a> {
    pub fn for_journal(root_journal: &'a mut Journal) -> ManagedFileJournal {
        ManagedFileJournal {
            root_journal
        }
    }

    pub fn create_or_get_managed_file(&mut self, file_path: &PathBuf) -> ManagedFile  {
        let file_key_in_record = file_path.to_owned().into_os_string().into_string().unwrap();

        if !self.root_journal.contains_record(&file_key_in_record) {
            info!("The file at path {:?} is not managed (no entry found in the root journal).\
                 Creating a new managed directory", file_path);

            self.new_managed_file(file_path);
        }

        return self.get_managed_file(file_path).unwrap()
    }

    pub fn get_managed_file(&self, file_path: &PathBuf) -> Option<ManagedFile> {
        let file_key_in_record = file_path.to_owned().into_os_string().into_string().unwrap();

        if !self.root_journal.contains_record(&file_key_in_record) {
            None
        } else {
            let managed_file_record = self.root_journal.get_record(&file_key_in_record);

            Some(ManagedFile::new(
                file_key_in_record,
                managed_file_record.root.join(MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME),
                managed_file_record.root.to_owned()
            ))
        }
    }

    fn new_managed_file(&mut self, file_path: &PathBuf) -> PathBuf {
        let managed_file_actual_path = file_path.to_owned().into_os_string().into_string().unwrap();

        // Get a random string identifier for
        let mut t = term::stdout().unwrap();
        t.fg(term::color::BRIGHT_MAGENTA);
        t.attr(term::Attr::Bold);

        write!(t, "Hippo ").unwrap();
        t.reset();
        println!("Hippo is not managing {}, creating new journal", managed_file_actual_path);

        info!("The root hosted by this journal is at {:?}", self.root_journal.root);

        let dir_key = Uuid::new_v4().to_string();
        let managed_file_journal_dir = Path::new(&self.root_journal.root)
            .join(&dir_key);
        let managed_file_snaps_journal = Path::new(&self.root_journal.root)
            .join(&dir_key)
            .join(MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME);

        info!("Creating new managed directory for file {} at {:?}",
              managed_file_actual_path, managed_file_journal_dir);

        create_dir_all(&managed_file_journal_dir).expect("Could not probe (or create) journal directory for managed file");

        info!("Creating snaps journal for file {} at {:?}", managed_file_actual_path,
            managed_file_snaps_journal);

        OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(managed_file_snaps_journal)
            .unwrap();

        info!("Adding entry to root journal with key {} at location {:?}",
              managed_file_actual_path, managed_file_journal_dir);

        self.root_journal.add_record(file_path.to_owned(), managed_file_journal_dir.to_owned());

        return managed_file_journal_dir;
    }
}
