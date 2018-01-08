use std::path::PathBuf;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufReader, BufRead, Write};
use std::collections::HashMap;

pub struct SimpleRecord {
    pub key: String,
    pub value: String
}

pub trait MapsToSimpleRecord {
    fn from(simple_record: SimpleRecord) -> Self;
    fn to(&self) -> SimpleRecord;
}

#[derive(Debug)]
pub struct SimpleFileRecords<T: MapsToSimpleRecord> {
    dirty: bool,
    file_path: PathBuf,
    pub name: String,
    pub records: HashMap<String, T>
}

impl<T: MapsToSimpleRecord> Drop for SimpleFileRecords<T> {
    fn drop(&mut self) {
        self.write()
    }
}

impl<T: MapsToSimpleRecord> SimpleFileRecords<T> {
    pub fn new(record_set_name: String, file_path: PathBuf) -> Self {
        match file_path.parent() {
            Some(prefix) => {
                info!("Ensuring that a directory exists for {} (probing: {:?})", record_set_name, prefix);

                create_dir_all(prefix)
                    .expect(format!("Could not probe directory (or create) for journal {}", record_set_name).as_str());
            }

            None => {}
        }

        info!("Creating (or opening if already exists) for {} (probing: {:?})", record_set_name, file_path);

        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path).expect("Error opening/creating journal file");

        let file = BufReader::new(&f);
        let mut records = HashMap::new();

        for line in file.lines() {
            let line_string = line.unwrap();

            // This should be checked for an overall format, if the number could be processed,
            // because as of now this would totally work even for six spaces.
            if line_string.len() > 6 {
                let key_len = line_string[..6].parse::<usize>().unwrap();
                let key : String = line_string.chars().skip(6).take(key_len).collect();
                let value : String = line_string.chars().skip(6 + key_len).collect();

                records.insert(key.to_owned(), T::from(SimpleRecord {
                    key, value
                }));
            }
        }

        SimpleFileRecords { dirty: false, name: record_set_name, file_path, records }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, record: T) {
        self.dirty = true;
        self.records.insert(record.to().key, record);
    }

    pub fn get_record(&self, key: &String) -> &T {
        self.records.get(key).unwrap()
    }

    /*
    pub fn remove(&mut self, entry: T) {
        self.dirty = true;
        self.records.remove(entry)
    }
    */

    pub fn write(&self) {
        if !self.dirty {
            info!("No changes were made to record set `{}`, not writing anything to file", self.name);
        }

        let file_name_as_str = self.file_path.to_str().unwrap();

        let mut f = File::create(&self.file_path)
            .expect(format!("Could not open file to write {}", file_name_as_str).as_str());

        for (_, record) in &self.records {
            let simple_record : SimpleRecord = record.to();
            let mut writable_record = String::new();

            writable_record.push_str(format!("{:06}", simple_record.key.len()).as_str());
            writable_record.push_str(simple_record.key.as_str());
            writable_record.push_str(simple_record.value.as_str());
            writable_record.push_str("\n");

            f.write_all(writable_record.into_bytes().as_slice())
                .expect(format!("Could not write to file {}", file_name_as_str).as_str());
        }

        f.write_all(b"\n").expect(format!("Could not write to file {}", file_name_as_str).as_str());
    }
}
