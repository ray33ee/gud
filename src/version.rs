
use crate::metadata::{Metadata, VersionNumber, Contents};
use tar::{Builder, Header};
use std::fs::{File, OpenOptions};
use std::path::{PathBuf, Path};
use diffy::Patch;


pub struct Version {
    archive: Builder<File>,
    metadata: Metadata,
}

impl Version {
    pub fn new(path: & Path, version_number: VersionNumber, message: String) -> Self {

        let fp = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path).unwrap();

        let archive = Builder::new(fp);

        let metadata = Metadata::new(
            chrono::Utc::now(),
            version_number,
            message);

        Version {
            archive,
            metadata,
        }
    }

    pub fn append_patch(& mut self, patch: Patch<str>, path: & Path) {

        let raw_bytes = patch.to_string().bytes().collect::<Vec<_>>();

        let mut header = Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_size(raw_bytes.len() as u64);
        header.set_cksum();

        self.metadata.append_file(path, Contents::Patch);

        self.archive.append(&header, raw_bytes.as_slice()).unwrap();
    }

    pub fn append_snapshot(& mut self, path: & Path) {
        self.metadata.append_file(path, Contents::Snapshot);

        self.archive.append_path(path).unwrap();
    }

    pub fn finish(& mut self) {
        self.metadata.finish();

        let raw_bytes = serde_json::to_string(&self.metadata).unwrap();

        let mut header = Header::new_gnu();
        header.set_path(Path::new(".").join(".gud").join(".metadata")).unwrap();
        header.set_size(raw_bytes.len() as u64);
        header.set_cksum();

        let v: Vec<_> = raw_bytes.bytes().collect();

        self.archive.append(&header, v.as_slice()).unwrap();

        self.archive.finish().unwrap();

    }

}
