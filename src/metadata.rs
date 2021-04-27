
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use chrono::DateTime;
use std::fs::OpenOptions;
use tar::Archive;
use std::io::Read;
use std::panic::resume_unwind;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Contents {
    Snapshot,
    Patch
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionNumber {
    pub version: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    files: HashMap<PathBuf, Contents>,
    id: u128,
    created: String,
    version: VersionNumber,
    message: String,
}


impl Metadata {
    pub fn new(created: DateTime<chrono::Utc>, version: VersionNumber, message: String) -> Self {
        Metadata {
            files: HashMap::new(),
            id: 0,
            created: String::from(created.to_rfc2822()),
            version,
            message
        }

    }

    pub fn get_from_archive(path: & Path) -> Self {
        let fp = OpenOptions::new()
            .read(true)
            .open(path).unwrap();

        let mut archive = Archive::new(fp);

        let mut result = None;

        for entry in archive.entries().unwrap() {
            let entry = entry.unwrap();

            let file_path = entry.path().unwrap();

            if Path::new(".").join(file_path) == Path::new(".").join(".gud").join(".metadata") {

                let v = entry.bytes().map(|x| x.unwrap()).collect::<Vec<u8>>();

                result = Some(serde_json::from_reader(v.as_slice()).unwrap());
            }
        }

        result.unwrap()
    }

    pub fn append_file(& mut self, path: & Path, contents: Contents) {
        self.files.insert(PathBuf::from(path), contents);
    }

    pub fn finish(& mut self) {
        #[cfg(feature = "v4")] {
            self.id = uuid::Uuid::new_v4().as_u128()
        }
    }
}