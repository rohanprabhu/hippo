extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate users;

extern crate colored;
extern crate pretty_bytes;

use std::path::PathBuf;
use std::ops::Add;
use std::fs::copy;
use std::fs::metadata;
use std::{ffi, io};

use self::colored::*;
use self::pretty_bytes::converter::convert;

use self::users::{get_user_by_uid, get_current_uid};

use self::chrono::prelude::*;
use super::super::utils::simple_file_records::{SimpleRecord, SimpleFileRecords, MapsToSimpleRecord};

static AUTO_SNAPSHOT_NAME_FORMAT: &'static str = "%Y%m.%d.%H%M.%S";
static AUTO_SNAPSHOT_COMMENT_FORMAT: &'static str = "%a %b %e %T %Y";

#[derive(Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub snapshot_name: String,
    pub comment: String,
    pub created_time: DateTime<Utc>,
    pub relative_file_path: String,
    pub author: String
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
    Null
}

pub enum Snapshot<'a> {
    Synthetic(SyntheticSnapshot),
    Tangible(&'a SnapshotEntry)
}

pub struct ManagedFile<'a> {
    tangible_snapshot_journal: SimpleFileRecords<SnapshotEntry>,
    snapshot_storage: &'a PathBuf,
    target_file: &'a PathBuf
}

pub struct SnapshotsListing<'a> {
    pub snapshots: Vec<Snapshot<'a>>,
    pub synthetic_count: u8,
    pub tangible_count: u8
}

pub struct ManagedFileError;

impl From<ffi::OsString> for ManagedFileError {
    fn from(_: ffi::OsString) -> Self { ManagedFileError {} }
}

impl From<io::Error> for ManagedFileError {
    fn from(_: io::Error) -> Self { ManagedFileError {} }
}

impl <'a> ManagedFile<'a> {
    pub fn new(file_key: &'a PathBuf, snapshot_journal_file: PathBuf, snapshot_storage: &'a PathBuf) -> ManagedFile<'a> {
        ManagedFile {
            tangible_snapshot_journal: SimpleFileRecords::new(
                format!("snapshot_journal({})", file_key.to_str().unwrap()),
                snapshot_journal_file
            ),
            snapshot_storage,
            target_file: file_key
        }
    }

    pub fn snap_current_state(
        &mut self, snapshot_name: Option<String>, comment: Option<String>, author: Option<String>
    ) -> Result<(), ManagedFileError> {
        let created_time  = Utc::now();
        let created_local_time = created_time.with_timezone(&Local);
        let author  = author.unwrap_or(get_user_by_uid(get_current_uid()).unwrap().name().to_string()).to_owned();
        let comment = comment.unwrap_or(format!(
            "Created snapshot by {} on {}",
            author,
            created_local_time.format(AUTO_SNAPSHOT_COMMENT_FORMAT)
        )).to_owned();

        let snapshot_name = snapshot_name.unwrap_or(
                created_local_time.format(AUTO_SNAPSHOT_NAME_FORMAT
            ).to_string())
            .to_owned();

        let snapped_file_name = self.target_file.file_name()
            .unwrap()
            .to_os_string()
            .into_string()?
            .add("-")
            .add(snapshot_name.as_str());

        info!("Copying target file {:?} into directory", self.target_file);

        copy(
            &self.target_file,
            self.snapshot_storage.join(&snapped_file_name)
        )?;

        info!("Updating managed file journal");

        println!("{} Created snapshot {} for {} (Size: {})",
           "OK".green().bold(),
            snapshot_name.as_str().cyan(),
            self.target_file.to_owned().into_os_string().into_string().unwrap().as_str().bold(),
            convert(metadata(&self.target_file).unwrap().len() as f64)
        );

        self.tangible_snapshot_journal.add(SnapshotEntry {
            snapshot_name,
            relative_file_path: snapped_file_name,
            comment,
            created_time,
            author
        });

        Ok(())
    }

    pub fn get_snapshots(&self) -> SnapshotsListing {
        let mut snapshots = Vec::<Snapshot>::new();
        let mut synthetic_count : u8 = 0;
        let mut tangible_count : u8 = 0;

        snapshots.push(Snapshot::Synthetic(SyntheticSnapshot::Null));
        synthetic_count = synthetic_count + 1;

        let mut tangible_entries : Vec<&SnapshotEntry> = self.tangible_snapshot_journal.records
            .values()
            .collect();

        tangible_entries.sort_by(|a, b|
            a.created_time.cmp(&b.created_time)
        );

        for snapshot_entry in tangible_entries {
            snapshots.push(Snapshot::Tangible(snapshot_entry));
            tangible_count = tangible_count + 1;;
        }

        return SnapshotsListing {
            snapshots,
            synthetic_count,
            tangible_count
        }
    }
}

