use std::fs::read_to_string;
use diffy::{create_patch, PatchFormatter};
use std::collections::HashMap;
use std::path::Path;



enum Contents {
    Snapshot,
    Patch
}

struct Project<'p> {
    versions: Vec<Version<'p>>,
    name: & 'p str,

}

struct Version<'v> {
    author: & 'v str,
    number: [u32; 3],
    id: & 'v str,
    data: HashMap<& 'v Path, Contents>,
}

fn create_repo(path: & Path) {

}

fn main() {
    let a = read_to_string(".\\diff\\a.txt").unwrap();
    let b = read_to_string(".\\diff\\b.txt").unwrap();

    let patch = create_patch(&a, &b);

    let f = PatchFormatter::new().with_color();
    print!("{}", f.fmt_patch(&patch));
}
