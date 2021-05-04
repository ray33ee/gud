#![feature(seek_stream_len)]
#![feature(iter_advance_by)]

mod archive;
mod walk_repo;

use archive::{Archive, VersionNumber};
use std::fs::{File, OpenOptions};
use std::io::{Read, copy};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;
use diffy::{create_patch, Patch, DiffOptions};
use std::str::{from_utf8_unchecked, from_utf8};
use std::ops::Deref;


fn main() {

    use std::env::{current_dir, set_current_dir};

    let current = current_dir().unwrap();

    set_current_dir("E:\\Software Projects\\IntelliJ\\gud\\diff").unwrap();

    let mut archive = Archive::new(Path::new(".").join(".gud").join(".versions"));

    //archive.create_repo(VersionNumber{ number: 100 }, String::from("initial commit"));

    //archive.commit_version(VersionNumber{ number: 101 }, String::from("Version 1"));

    //archive.commit_version(VersionNumber{ number: 102 }, String::from("Version 2"));

    //archive.commit_version(VersionNumber{ number: 104 }, String::from("V3"));

    let mut reader = archive.reader();

    //reader.file(0, ".\\a.txt").unwrap();

    reader.revert(3);




    set_current_dir(current).unwrap();



}
