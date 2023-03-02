//! Invoke and parse the output of the `git status` command.
//!
//! Exports the `status` function, which invokes `git status` and parses its
//! output into a `Status` structure.

use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusTag {
    Modified,
    Deleted,
    Untracked,
}

impl std::fmt::Display for StatusTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StatusTag::Modified  => "~",
            StatusTag::Deleted   => "-",
            StatusTag::Untracked => "+",
        })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct StatusEntry {
    tag: StatusTag,
    path: PathBuf,
}

impl std::fmt::Display for StatusEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f, " [{}] {}",
            self.tag,
            self.path.to_str().expect("Path must be valid unicode")
        )
    }
}

/// A data structure for `git status`'s output
///
/// Holds `StatusEntry` structs that can be either staged or unstaged.
#[derive(Default)]
pub struct Status {
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
}

impl Status {
    fn new() -> Self {
        Default::default()
    }
}

pub fn status() -> Result<Status, io::Error> {
    let output = Command::new("git")
                         .arg("status")
                         .arg("--porcelain")
                         .output();

    output
        .and_then(|output| String::from_utf8(output.stdout)
                  .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string())))
        .and_then(|string| parse_status(&string))
        .and_then(|mut status| {
            status.staged.sort();
            status.unstaged.sort();
            Ok(status)
        })
}

/// Parse an entry type from a `git status` tag string
///
/// # Examples
///
/// ```
/// let tag = "M";
/// let tag_type = parse_status_tag(tag);
///
/// assert_eq!(tag_type, StatusTag::Modified);
/// ```
fn parse_status_tag(tag: &str) -> Result<StatusTag, io::Error> {
    match tag {
        "M"  => Ok(StatusTag::Modified),
        "D"  => Ok(StatusTag::Deleted),
        "??" => Ok(StatusTag::Untracked),
        _    => Err(io::Error::new(io::ErrorKind::InvalidData,
                                   format!("Unknown `git status` tag \"{}\"", tag)))
    }
}

/// Parse a UTF-8 text as a `git status` line.
///
/// On success, returns the entry and a flag that indicates whether the entry
/// is staged (true) or unstaged (false).
fn parse_status_line(line: &str) -> Result<(StatusEntry, bool), io::Error> {
    let split: Vec<&str> = line.trim().splitn(2, ' ').collect();
    if split.len() != 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
                                  "String is not a valid `git status` line."));
    }

    let tag = split.get(0).expect("git status line must have a tag");
    let file = split.get(1).expect("git status line must have a file name");
    parse_status_tag(tag)
        .and_then(|tag| {
            let staged = !line.starts_with(' ') && tag != StatusTag::Untracked;
            Ok((
                StatusEntry{
                    tag,
                    path: PathBuf::from(file.trim())
                },
                staged
            ))
        })
}

/// Parse the output of `git status` into a `Status` struct.
fn parse_status(output: &str) -> Result<Status, io::Error> {
    let mut result: Status = Status::new();
    for line in output.split("\n").filter(|l| !l.is_empty()) {
        match parse_status_line(line) {
            Ok((entry, true))  => result.staged.push(entry),
            Ok((entry, false)) => result.unstaged.push(entry),
            Err(err)           => return Err(err)
        }
    }

    return Ok(result);
}

pub fn commit() {
    println!("Called git commit!");
}
