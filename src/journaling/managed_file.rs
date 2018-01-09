extern crate term;
extern crate serde;
extern crate serde_json;

extern crate chrono;
extern crate users;

extern crate colored;
extern crate pretty_bytes;

use std::path::{Path, PathBuf};
use std::ops::Add;
use std::fs::copy;
use std::fs::metadata;

use pretty_bytes::converter::convert;

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
    TangibleSnapshot(&'a SnapshotEntry)
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

    pub fn snap_current_state(&mut self, snapshot_name: Option<String>, comment: Option<String>, author: Option<String>) {
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
            .into_string().unwrap()
            .add("-")
            .add(snapshot_name.as_str());

        info!("Copying target file {:?} into directory", self.target_file);

        copy(
            &self.target_file,
            self.snapshot_storage.join(&snapped_file_name)
        ).expect("Failed to copy file, could not write to destination in journal root");

        info!("Updating managed file journal");

        let mut t = term::stdout().unwrap();
        t.fg(term::color::BRIGHT_GREEN).unwrap();
        println!("{} Created snapshot {} for {} (Size: {}B)", )
        write!(t, "OK ");
        t.reset();

        write!(t, "Created snapshot ");
        t.fg(term::color::BRIGHT_CYAN);
        t.attr(term::Attr::Bold);
        write!(t, "{} ", snapshot_name);
        t.reset();

        writeln!(t, "for {} (Size: {}B)",
            self.target_file.to_owned().into_os_string().into_string().unwrap(),
            metadata(&self.target_file).unwrap().len()
        );

        self.tangible_snapshot_journal.add(SnapshotEntry {
            snapshot_name,
            relative_file_path: snapped_file_name,
            comment,
            created_time,
            author
        });
    }

    pub fn get_snapshots(&self) -> Vec<Snapshot> {
        let mut snapshots = Vec::<Snapshot>::new();
        snapshots.push(Snapshot::Synthetic(SyntheticSnapshot::Null));

        let mut tangible_entries : Vec<&SnapshotEntry> = self.tangible_snapshot_journal.records
            .values()
            .collect();

        tangible_entries.sort_by(|a, b|
            a.created_time.cmp(&b.created_time)
        );

        for snapshot_entry in tangible_entries {
            snapshots.push(Snapshot::TangibleSnapshot(snapshot_entry))
        }

        return snapshots;
    }
}

