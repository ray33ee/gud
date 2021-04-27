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
- Automatic version incrementing
  - Version will increment, but can be changed by user too
- When creating a new repo, add optional ability to follow a wizard
    - Docs:
      - changelog - Use [this](https://keepachangelog.com/en/1.0.0/) template 
      - licence - Chose from a list of licences
      - readme.md - Very basic readme.md template
      - .gudignore - Empty file  
- Figure out how to classify files as text or binary
    - Text based files are stored as patch sets
    - Binary files are stored as snapshots
    - Maybe create a sophisticated class for 
      - identifying file types
      - creating patches for text and binary types
- use .gudignore to allow user to selectively ignore files in a repo
- Automatically add version number to changelog when committing
- Change serialisation of Patches and metadata back to binary 
- Make sure the version ID is created properly

### Unfinished Ideas
- Allow the arbitrary changing of version metadata (version number, version message, etc)
  - How will this work with the version ID?

## [0.1.1] - 2021-04-25
### Added
- A `Version` to help creation of the version archive
- A `Metadata` struct to contain information about the version
- `create_repo` and `commit_version` lay the groundwork for a basic VCS
- Version control creates .first and .last snapshots and versions (comprised of patches and snapshots)
- Each repo contains
  - First version (snapshot)
  - Last version (snapshot)
  - A set of all commited versions (snapshots and patches)

## [0.1.0] - 2021-04-25
### Added
- Initial Commit