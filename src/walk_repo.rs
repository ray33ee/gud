use std::iter::Filter;
use walkdir::FilterEntry;

struct RepoWalker {
    walker: FilterEntry<walkdir::IntoIter, P>
}