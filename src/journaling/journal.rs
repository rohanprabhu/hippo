extern crate chrono;

use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::collections::HashMap;

use self::chrono::prelude::*;
use super::super::utils::simple_file_records::{SimpleRecord, SimpleFileRecords, MapsToSimpleRecord};

static ROOT_JOURNAL_FILE_NAME: &'static str = "__hippo_journal";
static MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME: &'static str = "__snaps_journal";
static DEFAULT_SNAPSHOT_TIME_FORMAT: &'static str = "%Y-%m-%d-%H:%M.%S";

pub struct SnapshotEntry {
    snapshot_name: String,
    comment: String,
    created_time: DateTime<Local>,
    relative_file_path: PathBuf
}

impl MapsToSimpleRecord for SnapshotEntry {
    fn from(_: SimpleRecord) -> Self {
        unimplemented!()
    }

    fn to(&self) -> SimpleRecord {
        unimplemented!()
    }
}

pub enum SyntheticSnapshot {
    Live,
    Null
}

pub enum Snapshot {
    Synthetic(SyntheticSnapshot),
    TangibleSnapshot(SnapshotEntry)
}

pub struct ManagedFile {
    pub snapshots: HashMap<String, Snapshot>,
    tangible_snapshot_journal: SimpleFileRecords<SnapshotEntry>
}

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
    root: PathBuf,
    pub root_journal_config: SimpleFileRecords<RootJournalEntry>
}

impl Journal {
    pub fn initialize(root: String) -> Journal {
        let root_str = root.as_str().to_owned();

        info!("Loading journal into prefix {}", root);

        // Create the initial journal if it is not found
        Journal::initialize_journal_root(&root_str);

        let root_journal : SimpleFileRecords<RootJournalEntry> = SimpleFileRecords::load(
            String::from("root_journal"),
            Journal::get_root_journal_path(&root_str)
        );

        info!("Found entries in root journal {:?}", root_journal.records);

        Journal {
            root: Path::new(&root).to_path_buf(),
            root_journal_config: root_journal
        }
    }

    pub fn get_managed_file(&self, key: String) -> Option<ManagedFile> {
        if self.root_journal_config.records.contains_key(&key) {
            Some(ManagedFile {
                snapshots: HashMap::new(),
                tangible_snapshot_journal: SimpleFileRecords::load(
                    format!("journal({})", key),
                    self.root.join(
                    Path::join(
                        &self.root_journal_config.records.get(&key).unwrap().root,
                        MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME
                    )
                ).to_path_buf())
            })
        } else {
            None
        }
    }

    fn get_root_journal_path(root: &str) -> PathBuf {
        Path::new(root).join(ROOT_JOURNAL_FILE_NAME)
    }

    fn initialize_journal_root(root: &str) {
        let f = Path::new(&root);

        if f.exists() {
            if f.is_dir() {
                info!("Found a directory at {}", root);
            } else {
                error!("The file exists, but is not a directory. Delete the node at {} or \
                            use a different journal root. Cannot initialize", root);
            }
        } else {
            info!("No directory found at {}. Attempting to initialize", root);
            Journal::create_empty_journal(root);
        }
    }

    fn create_empty_journal(root: &str) {
        Journal::create_journal_directory(root);
        Journal::create_empty_root_journal(root);
    }

    fn create_empty_root_journal(root: &str) {
        let root_journal_path = Journal::get_root_journal_path(root);

        if !root_journal_path.exists() {
            info!("Created new root journal at {}", &root_journal_path.to_str().unwrap());
            let root_journal = File::create(&root_journal_path);

            if let Err(_) = root_journal {
                panic!("Could not create root journal for at {}", &root_journal_path.to_str().unwrap());
            }
        } else {
            info!("Root journal already exists at {}", &root_journal_path.to_str().unwrap());
        }
    }

    fn create_journal_directory(root: &str) {
        let f = fs::create_dir(root);

        if f.is_err() {
            panic!(f)
        } else {
            println!("Created new journal root at {}", root);
        }
    }
}
