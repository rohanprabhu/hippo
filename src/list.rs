extern crate chrono;
extern crate colored;

use std::path::PathBuf;
use std::io::Error;

use journaling::journal::Journal;
use journaling::managed_file::{Snapshot, SnapshotEntry, SyntheticSnapshot};
use journaling::managed_file_journal::*;

use prettytable::Table;
use prettytable::format::{FormatBuilder, TableFormat};

use self::colored::*;
use self::chrono::prelude::*;

static DEFAULT_SNAPSHOT_TIME_FORMAT: &'static str = "%a %b %e %T %Y";

//lazy_static! {
// No clue why lazy_static! is not working, will fix it.
fn hippo_list_display_table_format() -> TableFormat {
    FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(0, 1)
        .build()
}
//    };
//}

pub struct ListError;

impl From<Error> for ListError {
    fn from(_: Error) -> Self {
        ListError {}
    }
}

pub fn list(journal: &mut Journal, file_paths: Vec<PathBuf>) -> Result<(), ListError> {
    let managed_file_journal = ManagedFileJournal::for_journal(journal);
    let mut start = true;

    for file_path in file_paths {
        let file_path_string = file_path.to_owned().into_os_string().into_string()
            .expect("Could not read the file path, it might contain invalid characters");

        info!("Probing for snapshots for {:?}", file_path);

        let managed_file = managed_file_journal.get_managed_file(&file_path);

        if !start { println!(); } else { start = false; }

        match managed_file {
            Some(managed_file) => {
                let snapshots_listing = managed_file.get_snapshots();

                println!("{} {}, {} snapshots (+{} synthetic)", "OK".green(), file_path_string,
                         snapshots_listing.tangible_count, snapshots_listing.synthetic_count);
                println!();

                print_snapshot_table(snapshots_listing.snapshots);
            }

            None => {
                println!("{} {}, file not managed by {}", "WARN".red(), file_path_string, "hippo".magenta());
            }
        }
    }

    Ok(())
}

fn print_snapshot_table(snapshots: Vec<Snapshot>) {
    let mut table = Table::new();
    table.add_row(row![b->"Name", b->"Comment", b->"Author", b->"Creation Time"]);

    for snapshot in snapshots {
        match snapshot {
            Snapshot::Synthetic(some) => process_table_for_synthetic(&mut table, &some),
            Snapshot::Tangible(entry) => process_table_for_snapshot_entry(&mut table, &entry)
        }
    }

    table.set_format(hippo_list_display_table_format());
    table.print_tty(true);
}

fn process_table_for_synthetic(table: &mut Table, synthetic_snapshot: &SyntheticSnapshot) {
    match synthetic_snapshot {
        &SyntheticSnapshot::Null => {
            table.add_row(row!["(null)", "(synthetic snapshot; noent inode)", "", "Before all time"]);
        }
    }
}

fn process_table_for_snapshot_entry(table: &mut Table, snapshot_entry: &SnapshotEntry) {
    let local_creation_date = snapshot_entry.created_time.with_timezone(&Local)
        .format(DEFAULT_SNAPSHOT_TIME_FORMAT).to_string();

    table.add_row(row![snapshot_entry.snapshot_name, snapshot_entry.comment,
        snapshot_entry.author, local_creation_date]);
}
