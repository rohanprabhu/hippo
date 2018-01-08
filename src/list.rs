extern crate chrono;
extern crate term;

use std::path::PathBuf;
use std::io::Error;

use self::term::StdoutTerminal;

use journaling::journal::Journal;
use journaling::managed_file::Snapshot;
use journaling::managed_file::SyntheticSnapshot;
use journaling::managed_file_journal::*;

use prettytable::Table;
use prettytable::format::{FormatBuilder, TableFormat};

use self::chrono::prelude::*;

static DEFAULT_SNAPSHOT_TIME_FORMAT: &'static str = "%a %b %e %T %Y";

pub struct HippoList {
    format: TableFormat,
    out: Box<StdoutTerminal>
}

pub struct ListError;

impl From<term::Error> for ListError {
    fn from(_: term::Error) -> Self {
        ListError {}
    }
}

impl From<Error> for ListError {
    fn from(_: Error) -> Self {
        ListError {}
    }
}

impl HippoList {
    pub fn new() -> HippoList {
        HippoList {
            format: FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(0, 1)
                .build(),
            out: term::stdout().unwrap()
        }
    }

    pub fn list(&mut self, journal: &mut Journal, file_paths: Vec<PathBuf>) -> Result<(), ListError> {
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
                    let snapshots = managed_file.get_snapshots();

                    self.out.fg(term::color::BRIGHT_GREEN)?;
                    self.out.attr(term::Attr::Bold)?;
                    write!(self.out, "OK ")?;

                    self.out.reset()?;
                    writeln!(self.out, "{}, {} snapshots", file_path_string, snapshots.len())?;
                    writeln!(self.out)?;

                    self.print_snapshot_table(managed_file.get_snapshots());
                }

                None => {
                    self.out.fg(term::color::BRIGHT_RED)?;
                    self.out.attr(term::Attr::Bold)?;
                    self.out.reset()?;

                    write!(self.out, "WARN ")?;
                    writeln!(self.out, "{}, file not managed by hippo", file_path_string)?;
                }
            }
        }

        Ok(())
    }

    fn print_snapshot_table(&self, snapshots: Vec<Snapshot>) {
        let mut table = Table::new();
        table.add_row(row![b->"Name", b->"Comment", b->"Author", b->"Creation Time"]);
        table.add_row(row!["    ", "    " , "    ", "    "]);

        for snapshot in snapshots {
            match snapshot {
                Snapshot::Synthetic(some) => {
                    match some {
                        SyntheticSnapshot::Null => {
                            table.add_row(row!["(null)", "(synthetic snapshot; noent inode)", "", "Before all time"]);
                        }

                        /*
                    SyntheticSnapshot::Live => {
                        table.add_row(row!["(live)", "(synthetic snapshot; edits made)", "", "Absolute current moment"]);
                    }*/
                    }
                }

                Snapshot::TangibleSnapshot(entry) => {
                    let local_creation_date = entry.created_time.with_timezone(&Local)
                        .format(DEFAULT_SNAPSHOT_TIME_FORMAT).to_string();

                    table.add_row(row![entry.snapshot_name, entry.comment, entry.author, local_creation_date]);
                }
            }
        }

        table.set_format(self.format);
        table.print_tty(true);
    }
}
