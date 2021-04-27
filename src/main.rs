mod version;
mod metadata;

use std::fs::{read_to_string};
use diffy::{create_patch};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tar::{Archive, Builder};
use std::fs::OpenOptions;

use walkdir::{WalkDir};
use crate::metadata::VersionNumber;
use crate::version::Version;
use std::io::{Seek, SeekFrom, Read};

fn create_snapshot_archive(path: & Path) -> std::io::Result<()> {

    let fp = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path).unwrap();

    let mut builder = Builder::new(fp);

    for entry in WalkDir::new(".").into_iter().filter_entry(|dir| {

        dir.path() != Path::new(".").join(".gud")
    })
        .into_iter()
        .filter_map(|e| e.ok()) {

        let file_path = entry.path();

        if file_path.is_file() {

            builder.append_path(file_path);

        }


    }

    builder.finish();

    Ok(())
}

fn create_repo(path: & Path, first_version: VersionNumber) -> std::io::Result<()> {
    use std::env::{current_dir, set_current_dir};

    let current = current_dir().unwrap();

    set_current_dir(path).unwrap();

    std::fs::create_dir(Path::new(".gud"))?;

    std::fs::create_dir(Path::new(".gud").join("versions")).unwrap();

    create_snapshot_archive(Path::new(".gud").join(Path::new(".first")).as_path());

    commit_version(path, 0, String::from("Initial Commit"));

    set_current_dir(current).unwrap();

    Ok(())

}

fn is_text() -> bool {
    true
}

fn get_locations_from_archive(previous_version_path: & Path) -> HashMap<PathBuf, (u64, u64)> {
    let mut map = HashMap::new();

    let fp = OpenOptions::new()
        .read(true)
        .open(Path::new(".").join(previous_version_path)).unwrap();

    let mut previous_version = Archive::new(fp);

    //Add all the files in the previous version to a hashmap
    for entry in previous_version.entries().unwrap() {
        let entry = entry.unwrap();

        let path_copy = entry.path().unwrap();

        if path_copy.is_file() {
            map.insert(Path::new(".").join(path_copy), (entry.raw_file_position(), entry.size()));
        }
    }

    map
}

fn commit_version(path: & Path, number: u64, message: String) -> std::io::Result<()> {

    use std::env::{current_dir, set_current_dir};

    let current = current_dir().unwrap();

    set_current_dir(path).unwrap();

    let previous_version_path = Path::new(".gud").join(if path.join(Path::new(".gud").join(".last")).exists() {
        Path::new(".last")
    } else {
        Path::new(".first")
    });

    //Hash map containing all file paths and their locations within the archive
    let file_map = get_locations_from_archive(previous_version_path.as_path());

    println!("file map: {:?}", file_map);

    let mut version_archive = Version::new(
        Path::new(".").join(".gud").join("versions").join(format!("{}", number).as_str()).as_path(),
        VersionNumber { version: number },
        message
    );

    let mut previous_archive_fp = OpenOptions::new()
        .read(true)
        .open(Path::new(".").join(previous_version_path.as_path())).unwrap();

    for entry in WalkDir::new(".").into_iter().filter_entry(|dir| {

        dir.path() != Path::new(".").join(".gud")
    })
        .into_iter()
        .filter_map(|e| e.ok()) {

        let file_path = entry.path();

        if file_path.is_file() {

            let snapshot = if file_map.contains_key(file_path) {
                !is_text() || !is_text()

            } else {
                true
            };

            if snapshot {
                //Add the file directly to the version
                version_archive.append_snapshot(file_path);
            } else {
                //Compare files then add a patch to the version
                let (offset, size) = file_map.get(file_path).unwrap();

                let mut previous = String::new();
                let mut new_version = String::new();

                previous_archive_fp.seek(SeekFrom::Start(*offset));

                previous_archive_fp.by_ref().take(*size).read_to_string(& mut previous);

                let mut new_file = OpenOptions::new()
                    .read(true)
                    .open(file_path).unwrap();

                new_file.read_to_string(&mut new_version);

                version_archive.append_patch(diffy::create_patch(&previous, &new_version), file_path);
            }

        }


    }

    version_archive.finish();

    create_snapshot_archive(Path::new(".gud").join(Path::new(".last")).as_path());

    set_current_dir(current).unwrap();

    Ok(())
}

fn main() {

    //let f = PatchFormatter::new().with_color();
    //print!("{}", f.fmt_patch(&patch));

    //create_repo(Path::new("E:\\Software Projects\\IntelliJ\\gud\\diff"), VersionNumber { version: 1 }).unwrap();

    //let meta = metadata::Metadata::get_from_archive(Path::new("E:\\Software Projects\\IntelliJ\\gud\\diff\\.gud\\.first"));
    //println!("meta: {:?}", meta);

    commit_version(Path::new("E:\\Software Projects\\IntelliJ\\gud\\diff"), 3, String::from("Various changes and stuff")).unwrap();



    /*let fp = OpenOptions::new()
        .read(true)
        .open("E:\\Software Projects\\IntelliJ\\gud\\diff\\.gud\\.first").unwrap();

    let mut arc = Archive::new(fp);

    for entry in arc.entries().unwrap() {
        let e = entry.unwrap();
        println!("Path: {}", e.path().unwrap().display())
    }*/
}
