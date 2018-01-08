extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate whoami;

use std::path::{Path, PathBuf};
use std::ops::Add;
use std::fs::copy;

use self::chrono::prelude::*;
use super::super::utils::simple_file_records::{SimpleRecord, SimpleFileRecords, MapsToSimpleRecord};

static DEFAULT_SNAPSHOT_TIME_FORMAT: &'static str = "%Y-%m-%d-%H:%M.%S";
static AUTO_SNAPSHOT_NAME_FORMAT: &'static str = "%Y%m.%d.%H%M.%S";
static AUTO_SNAPSHOT_COMMENT_FORMAT: &'static str = "%a %b %e %T %Y";

#[derive(Serialize, Deserialize)]
pub struct SnapshotEntry {
    snapshot_name: String,
    comment: String,
    created_time: DateTime<Utc>,
    relative_file_path: String,
    author: String
}

impl MapsToSimpleRecord for SnapshotEntry {
    fn from(simple_record: SimpleRecord) -> Self {
        serde_json::from_str(simple_record.value.as_str())
            .expect("Invalid format while trying to read snapshot entry from journal")
    }

    fn to(&self) -> SimpleRecord {
        SimpleRecord {
            key: self.snapshot_name.to_owned(),
            value: serde_json::to_string(&self)
                .expect("Failed to serialize entry for snapshot (trying to write to journal)")
        }
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
    tangible_snapshot_journal: SimpleFileRecords<SnapshotEntry>,
    snapshot_storage: PathBuf,
    target_file: PathBuf
}

impl ManagedFile {
    pub fn new(file_key: String, snapshot_journal_file: PathBuf, snapshot_storage: PathBuf) -> ManagedFile {
        ManagedFile {
            tangible_snapshot_journal: SimpleFileRecords::new(
                format!("snapshot_journal({})", file_key),
                snapshot_journal_file
            ),
            snapshot_storage,
            target_file: Path::new(&file_key).to_path_buf()
        }
    }

    pub fn snap_current_state(&mut self, snapshot_name: Option<&String>, comment: Option<&String>, author: Option<&String>) {
        let created_time  = Utc::now();
        let created_local_time = created_time.with_timezone(&Local);
        let author  = author.unwrap_or(&whoami::username()).to_owned();
        let comment = comment.unwrap_or(&format!(
            "Created snapshot by {} on {}",
            author,
            created_local_time.format(AUTO_SNAPSHOT_COMMENT_FORMAT)
        )).to_owned();

        let snapshot_name = snapshot_name.unwrap_or(
                &created_local_time.format(AUTO_SNAPSHOT_NAME_FORMAT
            ).to_string())
            .to_owned();

        let snapped_file_name = self.target_file.file_name()
            .unwrap()
            .to_os_string()
            .into_string().unwrap()
            .add("-")
            .add(snapshot_name.as_str());

        info!("Copying target file {:?} into directory", self.target_file);

        copy(
            &self.target_file,
            self.snapshot_storage.join(&snapped_file_name)
        ).expect("Failed to copy file, could not write to destination in journal root");

        info!("Updating managed file journal");

        self.tangible_snapshot_journal.add(SnapshotEntry {
            snapshot_name,
            relative_file_path: snapped_file_name,
            comment,
            created_time,
            author
        });
    }
}

