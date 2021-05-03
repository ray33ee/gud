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
use crate::walk_repo::RepoWalker;

fn create_repo(archive: & mut Archive, number: VersionNumber, message: String) {

    std::fs::create_dir(Path::new(".").join(".gud")).unwrap();

    archive.create();

    let mut appender = archive.appender(number, message);

    for  file_path in RepoWalker::new(".").map(|x| x.unwrap())   {

        println!("file: {}", file_path.to_str().unwrap());

        appender.append_snapshot(file_path.as_path());

    }

    appender.finish();

    create_snapshot();
}

fn create_snapshot() {

    let fp = OpenOptions::new()
        .create(true)
        .write(true)
        .open(Path::new(".").join(".gud").join(".last")).unwrap();

    let mut zip = zip::ZipWriter::new(fp);

    for  file_path in RepoWalker::new(".").map(|x| x.unwrap())   {

        println!("file: {}", file_path.to_str().unwrap());

        zip.start_file(file_path.to_str().unwrap(), FileOptions::default()).unwrap();

        copy(& mut File::open(file_path).unwrap(), & mut zip).unwrap();

    }
}

fn commit_version(mut archive: & mut Archive, number: VersionNumber, message: String) {

    //Iterate over each file in the repo
    //    If the file exists in .last AND is suitable for use with diffy, compute a patch and push it. Otherwise push a snapshot
    //    Add each file to a .last.new archive
    //delete the old .last
    //rename .last.new to .last

    let mut appender = archive.appender(number, message);

    let mut fp = OpenOptions::new()
        .read(true)
        .open(Path::new(".").join(".gud").join(".last")).unwrap();

    let mut last = zip::ZipArchive::new(fp).unwrap();

    let mut diff_options = DiffOptions::new();

    diff_options.set_context_len(0);

    for file_path in RepoWalker::new(".").map(|x| x.unwrap()) {
        println!("file: {}", file_path.to_str().unwrap());

        let mut repo_fp = OpenOptions::new()
            .read(true)
            .open(file_path.as_path()).unwrap();

        if let Ok(mut data) = last.by_name(file_path.to_str().unwrap()) {
            let mut zip_str_buffer = String::new();
            let mut repo_str_buffer = String::new();

            data.read_to_string(&mut zip_str_buffer).unwrap();

            repo_fp.read_to_string(&mut repo_str_buffer).unwrap();

            let patch = diff_options.create_patch(&zip_str_buffer, &repo_str_buffer);

            println!("Patch: {}", patch);

            appender.append_patch(file_path.as_path(), &patch);

            println!("Original: {}", zip_str_buffer);
            println!("     New: {}", repo_str_buffer);
        } else {
            //Commit as a snapshot
            appender.append_snapshot(file_path.as_path());
        }
    }

    create_snapshot();

    appender.finish();
}

fn main() {

    use std::env::{current_dir, set_current_dir};

    let current = current_dir().unwrap();

    set_current_dir("E:\\Software Projects\\IntelliJ\\gud\\diff").unwrap();

    let mut archive = Archive::new(Path::new(".").join(".gud").join(".versions"));

    //create_repo(& mut archive, VersionNumber{ number: 100 }, String::from("initial commit"));

    //commit_version(& mut archive, VersionNumber{ number: 101 }, String::from("Version 1"));

    //commit_version(& mut archive, VersionNumber{ number: 102 }, String::from("Version 2"));

    let mut reader = archive.reader();

    reader.file(2, ".\\a.txt").unwrap();


    set_current_dir(current).unwrap();



}
