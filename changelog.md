# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- Use a TUI library for a more friendly interface
  - Colours red/green/etc. when showing file differences
  - Calling 'gud' on its own will bring up TUI
    - Calling with sub commands will execute those commands then return
- Support showing the files, differences, and metadata from any given version
    - Allow the user to choose their own text editor
- Each version gets a unique ID (a hash based on author name, project name, commit time, version message, etc.)
- Store snapshot of original and work on patch differences.
    - The last committed version should be stored as a snapshot
- Each version must have a version number
  - Automatic version incrementing
    - Version will increment, but can be changed by user too
- When creating a new repo, add optional ability to follow a wizard
    - Docs:
      - changelog - Use [this](https://keepachangelog.com/en/1.0.0/) template 
      - licence - Chose from a list of licences
      - readme.md - Very basic readme.md template
- When we commit a version, all files are committed (none of this stage to commit bs)    
- Use a database of all file extensions and whether they are text based or binary.
    - Text based files are stored as patch sets
    - Binary files are stored as snapshots
- use .gudignore to allow user to selectively ignore files in a repo
- Automatically add version number to changelog when committing
- Keep hashes of all files 
- Each file gets a unique identifier (based on the version in which it was added, its path and name)
- Use diffy (for patches, applying patches, and merging), tar (for archiving the head and tail versions)
- The latest version always stores the patches AND snapshots
    - When we add a new version, we 
        - Compare the new version with the top snapshot to obtain the patches
        - Replace the top snapshot with the new version
- Each repo contains
    - The original snapshot
    - The top snapshot
    - A list of versions comprising a set of patches/snapshots for each file

### Unfinished Ideas
- Allow the arbitrary changing of version metadata (version number, version message, etc)
  - How will this work with the version ID?

## [0.1.0] - 2021-04-25
### Added
- Initial Commit