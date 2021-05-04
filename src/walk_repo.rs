use std::iter::Filter;
use walkdir::{FilterEntry, WalkDir};
use std::path::{PathBuf, Path};
use std::error::Error;

pub struct RepoWalker {
    walker: walkdir::IntoIter,
    ignore_directories: bool
}

impl RepoWalker {
    pub fn new<P: AsRef<Path>>(repo_path: P, ignore_directories: bool) -> Self {

        //ToDo: When a RepoWalker instance is created, search for a .gudignore and add its rules to the ignore list

        RepoWalker {
            walker: WalkDir::new(repo_path).into_iter(),
            ignore_directories,
        }
    }
}

impl Default for RepoWalker {
    fn default() -> Self {
        Self::new(Path::new("."), true)
    }
}

impl Iterator for RepoWalker {
    type Item = Result<PathBuf, String>;

    fn next(& mut self) -> Option<Self::Item> {
        loop {
            let dent = match self.walker.next() {
                None => return None,
                Some(result) => match result {
                    Ok(v) => v,
                    Err(err) => return Some(Err(err.to_string())),
                },
            };
            //Skip the .gud directory
            if dent.path() == Path::new(".").join(".gud") {
                if dent.path().is_dir() {
                    self.walker.skip_current_dir();
                }
                continue;
            }
            //If we are only interested in walking through the files skip directories
            if self.ignore_directories && dent.path().is_dir() {
                continue;
            }
            return Some(Ok(dent.into_path()));
        }
    }
}