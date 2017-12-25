use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::collections::HashMap;

pub trait MirrorsKvEntry {
    fn to_kv_entry(&self) -> KvEntry;
    fn from_kv_entry(kv_entry: KvEntry) -> Self;
}

pub struct KvEntry {
    pub key: String,
    pub value: String
}

pub struct SplitKvConfig<T: MirrorsKvEntry> {
    dirty: bool,
    file_path: PathBuf,
    pub entries: HashMap<String, T>
}

impl<T: MirrorsKvEntry> Drop for SplitKvConfig<T> {
    fn drop(&mut self) {
        self.write()
    }
}

impl<T: MirrorsKvEntry> SplitKvConfig<T> {
    pub fn load(file_path: PathBuf) -> Self {
        let f = File::open(&file_path).expect(format!("Cannot open file {}", file_path.to_str().unwrap()).as_str());
        let file = BufReader::new(&f);
        let mut entries = HashMap::new();

        for line in file.lines() {
            let line_string = line.unwrap();

            // This should be checked for an overall format, if the number could be processed,
            // because as of now this would totally work even for six spaces.
            if line_string.len() > 6 {
                let key_len = line_string[..6].parse::<usize>().unwrap();
                let key : String = line_string.chars().skip(6).take(key_len).collect();
                let value : String = line_string.chars().skip(6 + key_len).collect();

                entries.insert(key.to_owned(), T::from_kv_entry(KvEntry {
                    key, value
                }));
            }
        }

        SplitKvConfig { dirty: false, file_path, entries }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, entry: T) {
        self.dirty = true;
        self.entries.insert(entry.to_kv_entry().key, entry);
    }

    pub fn write(&self) {
        if !self.dirty {
            info!("No changes were made to the split-kv-config, not writing anything to file");
        }

        let file_name_as_str = self.file_path.to_str().unwrap();

        let mut f = File::create(&self.file_path)
            .expect(format!("Could not open file to write {}", file_name_as_str).as_str());

        for (_, entry) in &self.entries {
            let kv_entry : KvEntry = entry.to_kv_entry();
            let mut writable_kv_entry = String::new();

            writable_kv_entry.push_str(format!("{:06}", kv_entry.key.len()).as_str());
            writable_kv_entry.push_str(kv_entry.key.as_str());
            writable_kv_entry.push_str(kv_entry.value.as_str());
            writable_kv_entry.push_str("\n");

            f.write_all(writable_kv_entry.into_bytes().as_slice())
                .expect(format!("Could not write to file {}", file_name_as_str).as_str());
        }

        f.write_all(b"\n").expect(format!("Could not write to file {}", file_name_as_str).as_str());
    }
}
