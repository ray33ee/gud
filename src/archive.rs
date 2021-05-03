

use std::fs::{File, OpenOptions};
use std::path::{PathBuf, Path};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use std::io::{Read, Write, Seek, SeekFrom};
use std::collections::{HashMap};
use lzma_rs::{lzma_compress, lzma_decompress};
use diffy::{Patch, apply_bytes};
use std::str::from_utf8;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Contents {
    Snapshot,
    Patch
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum FileType {
    File,
    Directory,
    SystemLink,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileFlags {
    flags: u8,
}

impl FileFlags {
    //Flags:
    //0-1 -> 0 = File, 1 = Directory, 2 = System Link
    //2 -> Read only
    //3 -> Modified time present
    //4 -> Accessed time present
    //5 -> Created time present

    fn new() -> Self {
        FileFlags {
            flags: 0
        }
    }

    fn set_file_type_file(& mut self) {
        //Do nothing
    }

    fn set_file_type_directory(& mut self) { self.flags |= 1; }

    fn set_file_type_link(& mut self) { self.flags |= 2; }

    fn set_read_only(& mut self, read_only: bool) { if read_only { self.flags |= 4; } }

    fn set_time(& mut self, time: & std::io::Result<SystemTime>, position: u8) {
        if time.is_ok() {
            self.flags |= position;
        }
    }

    fn set_modified_present(& mut self, modified_time: & std::io::Result<SystemTime>) { self.set_time(modified_time, 8); }

    fn set_accessed_present(& mut self, accessed_time: & std::io::Result<SystemTime>) { self.set_time(accessed_time, 16); }

    fn set_created_present(& mut self, created_time: & std::io::Result<SystemTime>) { self.set_time(created_time, 32); }

    fn get_nth_flag(& mut self, position: u8) -> bool { (self.flags & position) != 0 }

    fn get_file_type(& mut self) -> u8 { self.flags & 3 }

    fn get_read_only(& mut self) -> bool { self.get_nth_flag(4) }

    fn get_modified_present(& mut self) -> bool { self.get_nth_flag(8) }

    fn get_accessed_present(& mut self) -> bool { self.get_nth_flag(16) }

    fn get_created_present(& mut self) -> bool { self.get_nth_flag(32) }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Metadata {
    len: u64,
    flags: FileFlags,
    modified: SystemTime,
    accessed: SystemTime,
    created: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileHeader {
    compressed_size: u64,
    metadata: Metadata,
    path: PathBuf,
    contents: Contents,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionNumber {
    pub number: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionHeader {
    files: HashMap<PathBuf, u64>,
    number: VersionNumber,
    message: String,
}

impl VersionHeader {
    fn new(number: VersionNumber, message: String) -> Self {
        VersionHeader {
            files: HashMap::new(),
            number,
            message,
        }
    }

    fn insert(& mut self, path: & Path, offset: u64) {
        self.files.insert(PathBuf::from(path), offset);
    }
}

#[derive(Debug, Clone)]
struct Version {
    pub files: HashMap<PathBuf, (u64, FileHeader)>,
    pub number: VersionNumber,
    pub message: String,
}

impl Version {
    fn new(files: HashMap<PathBuf, (u64, FileHeader)>, number: VersionNumber, message: String) -> Self {

        Version {
            files,
            number,
            message,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionDirectory {
    directory: Vec<u64>,
}

impl VersionDirectory {
    pub fn new() -> Self {
        VersionDirectory {
            directory: Vec::new(),
        }
    }

    pub fn add(& mut self, offset: u64) { self.directory.push(offset); }
}

impl Metadata {
    fn new(path: & Path) -> Self {

        let metadata = std::fs::metadata(path).unwrap();

        let mut flags = FileFlags::new();

        if metadata.is_file() {
            flags.set_file_type_file();
        } else if metadata.is_dir() {
            flags.set_file_type_directory();
        }

        flags.set_read_only(metadata.permissions().readonly());

        flags.set_modified_present(&metadata.modified());
        flags.set_accessed_present(&metadata.accessed());
        flags.set_created_present(&metadata.created());

        Metadata {
            len: metadata.len(),
            flags,
            modified: metadata.modified().unwrap_or(SystemTime::now()), //Here we only use now() as a default value if no time is present. This value will not actually be used
            accessed: metadata.accessed().unwrap_or(SystemTime::now()),
            created: metadata.created().unwrap_or(SystemTime::now()),
        }
    }
}

impl FileHeader {
    fn new(path: & Path, contents: Contents) -> Self {
        let metadata = Metadata::new(path);
        let path = PathBuf::from(path);

        FileHeader {
            compressed_size: 0,
            metadata,
            path,
            contents
        }

    }
}

pub struct Archive {
    path: PathBuf,
}

impl Archive {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Archive {
            path: PathBuf::from(path.as_ref()),
        }
    }

    pub fn create(& self) {
        let fp = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path).unwrap();

        //insert a 8u64 at the beginning
        bincode::serialize_into(&fp, &8u64).unwrap();

        //Insert an empty VersionDirectory after
        bincode::serialize_into(&fp, &VersionDirectory::new()).unwrap();
    }

    pub fn appender(& mut self, number: VersionNumber, message: String) -> AppendArchive {
        AppendArchive::new(&self.path, number, message)
    }

    pub fn reader(& mut self) -> ReadArchive {
        ReadArchive::new(&self.path)
    }
}

pub struct AppendArchive {
    fp: File,
    backup_directory: VersionDirectory, //A backup of the version directory
    version_header: VersionHeader,
}

impl AppendArchive {
    //Open file
    fn new(archive_path: & Path, number: VersionNumber, message: String) -> Self {
        let mut fp = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(archive_path).unwrap();

        //Get the first u64 (version directory offset)
        let offset = bincode::deserialize_from::<_, u64>(&fp).unwrap();

        //seek to this offset
        fp.seek(SeekFrom::Start(offset)).unwrap();

        //make a backup of the data from offset to EOF
        let backup_directory = bincode::deserialize_from::<_, VersionDirectory>(&fp).unwrap();

        //Seek back to offset, so that future appends overwrite the old version directory
        fp.seek(SeekFrom::Start(offset)).unwrap();

        let version_header = VersionHeader::new(number, message);

        AppendArchive {
            fp,
            backup_directory,
            version_header,
        }

    }

    fn append_file<R: Read, P: AsRef<Path>>(& mut self, path: P, data: R, contents: Contents) {
        if path.as_ref().is_absolute() {
            panic!("Appended path MUST be relative.")
        }

        //Save the position of the header
        let position = self.fp.stream_position().unwrap();

        //Create the file header for the file entry
        let header = FileHeader::new(path.as_ref(), contents);

        //Write the header to the archive
        bincode::serialize_into(&self.fp, &header).unwrap();

        //Copy the file into the archive and compress it
        //    Move the compressed data and get the size of the data moved
        let start = self.fp.stream_position().unwrap();
        lzma_compress(& mut std::io::BufReader::new(data), & mut self.fp).unwrap();
        let compressed_size = self.fp.stream_position().unwrap() - start;

        //    Make a copy of the current seek position
        let save = self.fp.stream_position().unwrap();

        //    Go back and manually add the 'compressed_size' entry to the file header
        self.fp.seek(SeekFrom::Start(position)).unwrap();
        bincode::serialize_into(&self.fp, &compressed_size).unwrap();

        //    Seek back to the saved position
        self.fp.seek(SeekFrom::Start(save)).unwrap();

        //Add position of the header to list
        self.version_header.insert(path.as_ref(), position);
    }

    pub fn append_patch<P: AsRef<Path>>(& mut self, path: P, patch: & Patch<str>) {
        self.append_file(path, patch.to_string().as_bytes(), Contents::Patch);
    }

    //Append Version to archive, sort out directory and the directory offset
    pub fn append_snapshot<P: AsRef<Path>>(& mut self, path: P) {
        let fp = OpenOptions::new()
            .read(true)
            .open(path.as_ref()).unwrap();

        self.append_file(path, fp, Contents::Snapshot);
    }

    pub fn finish(& mut self) {

        let version_header_offset = self.fp.stream_position().unwrap();

        //Append the version header
        bincode::serialize_into(&self.fp, &self.version_header).unwrap();

        //Get the size of the file (offset version directory)
        let directory_offset = self.fp.stream_len().unwrap();

        //Add the new entry in the version directory
        self.backup_directory.add(version_header_offset);

        //append the new version directory
        bincode::serialize_into(&self.fp, &self.backup_directory).unwrap();

        //set the first u64 to the offset of the version directory
        self.fp.seek(SeekFrom::Start(0)).unwrap();

        bincode::serialize_into(&self.fp, &directory_offset).unwrap();
    }

}

pub struct ReadArchive {
    fp: File,
    version_headers: Vec<Version>,
}

impl ReadArchive {
    fn new(archive_path: & Path) -> Self {

        let mut fp = OpenOptions::new()
            .read(true)
            .open(archive_path).unwrap();

        //Get the first u64 (version directory offset)
        let version_directory_offset = bincode::deserialize_from::<_, u64>(&fp).unwrap();

        //Seek to directory
        fp.seek(SeekFrom::Start(version_directory_offset)).unwrap();

        //Get directory
        let version_directory = bincode::deserialize_from::<_, VersionDirectory>(&fp).unwrap();

        let mut version_headers = Vec::new();


        for offset in version_directory.directory.iter() {
            let mut file_header_map = HashMap::new();

            fp.seek(SeekFrom::Start(*offset)).unwrap();

            let header = bincode::deserialize_from::<_, VersionHeader>(&fp).unwrap();

            for (file_path, file_header_offset) in header.files.iter() {
                fp.seek(SeekFrom::Start(*file_header_offset)).unwrap();

                let file_head = bincode::deserialize_from::<_, FileHeader>(&fp).unwrap();

                file_header_map.insert(file_path.clone(), (fp.stream_position().unwrap(), file_head));

            }

            let version = Version::new(file_header_map, header.number, header.message.clone());

            version_headers.push(version);

        }

        ReadArchive {
            fp,
            version_headers,
        }
    }

    fn get_raw_file<W: Write, P: AsRef<Path>>(& self, version: usize, path: P, mut writer: & mut W) -> Option<()> {

        let mut fp = self.fp.try_clone().unwrap();

        let version = self.version_headers.get(version).unwrap();

        let (offset, header) = version.files.get(path.as_ref())?;

        let size = header.compressed_size;

        fp.seek(SeekFrom::Start(*offset)).unwrap();

        let mut taken = std::io::Read::by_ref(&mut fp).take(size);

        lzma_decompress(& mut std::io::BufReader::new(& mut taken), & mut writer).unwrap();

        Some(())
    }

    pub fn file<P: AsRef<Path>>(& mut self, version: usize, path: P) -> Option<String> {

        if version == self.version_headers.len() - 1 {
            //Just get the file from .last
        }

        let version_index = version  - {
            let mut index = 0;

            let mut it = self.version_headers.iter().rev();

            it.advance_by(self.version_headers.len() - version - 1).unwrap();

            for (i, version_header) in it.enumerate() {

                let (_, file_header) = version_header.files.get(&PathBuf::from(path.as_ref()))?;

                if let Contents::Snapshot = file_header.contents {
                    index = i;
                    break;
                }
            }

            println!("ind: {}", index);
            index
        };

        println!("Snapshot index: {}", version_index);

        //Now we have the index of the original snapshot for the file, keep getting and applying patches

        let mut it = self.version_headers.iter().take(version+1);

        it.advance_by(version_index+1);

        let mut previous = Vec::new();
        let mut patch = Vec::new();

        self.get_raw_file(version_index, path.as_ref(), & mut previous);

        for (i, version) in it.enumerate() {
            println!("repair: {}", version_index+i+1);
            self.get_raw_file(version_index+i+1, path.as_ref(), & mut patch);


            println!("patch: {}", from_utf8(&patch).unwrap());
            previous = apply_bytes(&previous, &Patch::from_bytes(&patch).unwrap()).unwrap();

            patch.clear();
        }

        println!("index: {}", from_utf8(&previous).unwrap());

        Some(String::new())
    }

}
