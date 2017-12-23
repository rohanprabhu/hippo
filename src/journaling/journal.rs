use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use super::super::utils::{KvEntry, SplitKvConfig, MirrorsKvEntry};

static ROOT_JOURNAL_FILE_NAME: &'static str = "__hippo_journal";
static MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME: &'static str = "__snaps_journal";

pub struct SnapshotEntry {

}

impl MirrorsKvEntry for SnapshotEntry {
    fn to_kv_entry(&self) -> KvEntry {
        unimplemented!()
    }

    fn from_kv_entry(_: KvEntry) -> Self {
        unimplemented!()
    }
}

pub struct ManagedFile {
    pub snapshot_journal: SplitKvConfig<SnapshotEntry>
}
pub struct MissingEntry {}

#[derive(Debug)]
pub struct RootJournalEntry {
    pub key: String,
    pub root: PathBuf
}

impl MirrorsKvEntry for RootJournalEntry {
    fn to_kv_entry(&self) -> KvEntry {
        KvEntry {
            key: self.key.to_owned(),
            value: String::from(self.root.to_str().unwrap())
        }
    }

    fn from_kv_entry(kv_entry: KvEntry) -> Self {
        RootJournalEntry {
            key: kv_entry.key,
            root: Path::new(&kv_entry.value).to_path_buf()
        }
    }
}

pub struct Journal {
    root: PathBuf,
    pub root_journal_config: SplitKvConfig<RootJournalEntry>
}

impl Journal {
    pub fn initialize(root: String) -> Journal {
        let root_str = root.as_str().to_owned();

        info!("Loading journal into prefix {}", root);

        // Create the initial journal if it is not found
        Journal::initialize_journal_root(&root_str);

        let root_journal : SplitKvConfig<RootJournalEntry> = SplitKvConfig::load(
            Journal::get_root_journal_path(&root_str)
        );

        println!("Found entries {:?}", root_journal.entries);

        Journal {
            root: Path::new(&root).to_path_buf(),
            root_journal_config: root_journal
        }
    }

    pub fn get_managed_file(&self, key: String) -> Result<ManagedFile, MissingEntry> {
        if self.root_journal_config.entries.contains_key(&key) {
            Ok(ManagedFile {
                snapshot_journal: SplitKvConfig::load(self.root.join(
                    Path::join(
                        &self.root_journal_config.entries.get(&key).unwrap().root,
                        MANAGED_FILE_SNAPSHOT_JOURNAL_FILE_NAME
                    )
                ).to_path_buf())
            })
        } else {
            Err(MissingEntry {})
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
