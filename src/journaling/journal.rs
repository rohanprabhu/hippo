extern crate chrono;

use std::path::{Path, PathBuf};

use super::super::utils::simple_file_records::{SimpleRecord, SimpleFileRecords, MapsToSimpleRecord};

static ROOT_JOURNAL_FILE_NAME: &'static str = "__hippo_journal";

#[derive(Debug)]
pub struct RootJournalEntry {
    pub key: String,
    pub root: PathBuf
}

impl MapsToSimpleRecord for RootJournalEntry {
    fn from(simple_record: SimpleRecord) -> Self {
        RootJournalEntry {
            key: simple_record.key.to_owned(),
            root: Path::new(&simple_record.value).to_path_buf()
        }
    }

    fn to(&self) -> SimpleRecord {
        SimpleRecord {
            key: self.key.to_owned(),
            value: String::from(self.root.to_str().unwrap())
        }
    }
}

pub struct Journal {
    pub root: PathBuf,
    pub root_journal_config: SimpleFileRecords<RootJournalEntry>
}

impl Journal {
    pub fn initialize(root: String) -> Journal {
        let root_str = root.as_str().to_owned();

        info!("Loading journal into prefix {}", root);

        let root_journal : SimpleFileRecords<RootJournalEntry> = SimpleFileRecords::new(
            String::from("root_journal"),
            Journal::get_root_journal_path(&root_str)
        );

        info!("Found entries in root journal {:?}", root_journal.records);

        Journal {
            root: Path::new(&root).to_path_buf(),
            root_journal_config: root_journal
        }
    }

    pub fn contains_record(&self, key: &String) -> bool {
        self.root_journal_config.records.contains_key(key)
    }

    pub fn get_record(&self, key: &String) -> &RootJournalEntry {
        self.root_journal_config.get_record(key)
    }

    pub fn add_record(&mut self, file_path: PathBuf, managed_root: PathBuf) {
        self.root_journal_config.add(RootJournalEntry {
            key: file_path.into_os_string().into_string().unwrap(),
            root: managed_root
        })
    }

    /*
    pub fn new_managed_file(&self, key: String) -> ManagedFile {
        return ManagedFile {
            snapshots: HashMap::new(),
            tangible_snapshot_journal: SimpleFileRecords::new(
                format!("journal({})", key),
                self.root.join(
                    Path::join(
                        &self.root_journal_config.records.get(&key).unwrap().root,
                        MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME
                    )
                ).to_path_buf())
        }
    }*/

    fn get_root_journal_path(root: &str) -> PathBuf {
        Path::new(root).join(ROOT_JOURNAL_FILE_NAME)
    }
}
