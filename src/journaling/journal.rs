extern crate chrono;

use std::path::{Path, PathBuf};
use std::collections::HashMap;

use self::chrono::prelude::*;
use super::super::utils::simple_file_records::{SimpleRecord, SimpleFileRecords, MapsToSimpleRecord};

static ROOT_JOURNAL_FILE_NAME: &'static str = "__hippo_journal";
static MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME: &'static str = "__snaps_journal";
static DEFAULT_SNAPSHOT_TIME_FORMAT: &'static str = "%Y-%m-%d-%H:%M.%S";

pub struct SnapshotEntry {
    snapshot_name: String,
    comment: String,
    created_time: DateTime<Utc>,
    relative_file_path: PathBuf
}

#[derive(Serialize, Deserialize)]
pub struct SerializableSnapshotEntry {
    snapshot_name: String,
    comment: String,
    created_time: i64,
    relative_file_path: String
}

impl SerializableSnapshotEntry {
    pub fn to_snapshot_entry(&self) -> SnapshotEntry {
        SnapshotEntry {
            snapshot_name: self.snapshot_name.to_owned(),
            comment: self.comment.to_owned(),
            created_time: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(self.created_time, 0),
                Utc),
            relative_file_path: Path::new(&self.relative_file_path).to_path_buf()
        }
    }

    pub fn from_snapshot_entry(snapshot_entry: SnapshotEntry) -> SerializableSnapshotEntry {
        SerializableSnapshotEntry {
            snapshot_name: snapshot_entry.snapshot_name,
            comment: snapshot_entry.comment,
            created_time: snapshot_entry.created_time.timestamp(),
            relative_file_path: String::from(snapshot_entry.relative_file_path.to_str().unwrap())
        }
    }
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

    pub fn contains_record(&self, key: String) -> bool {
        self.root_journal_config.records.contains_key(&key)
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
