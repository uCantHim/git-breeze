//! Invoke and parse the output of the `git status` command.
//!
//! Exports the `status` function, which invokes `git status` and parses its
//! output into a `Status` structure.

use std::io;
use std::process::Command;
use colored::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusTag {
    Unmodified,
    Modified,
    FileTypeChanged,
    Added,
    Deleted,
    Renamed,
    Copied,
    UpdatedUnmerged,

    Untracked,
    Ignored,
}

impl StatusTag {
    /// Parse an entry type from a `git status` tag character
    ///
    /// # Examples
    ///
    /// ```
    /// let tag = 'M';
    /// let tag_type = parse_status_tag(tag);
    ///
    /// assert_eq!(tag_type, StatusTag::Modified);
    /// ```
    fn from(tag: char) -> Option<StatusTag> {
        match tag {
            ' ' => Some(StatusTag::Unmodified),
            'M' => Some(StatusTag::Modified),
            'T' => Some(StatusTag::FileTypeChanged),
            'A' => Some(StatusTag::Added),
            'D' => Some(StatusTag::Deleted),
            'R' => Some(StatusTag::Renamed),
            'C' => Some(StatusTag::Copied),
            'U' => Some(StatusTag::UpdatedUnmerged),
            '?' => Some(StatusTag::Untracked),
            '!' => Some(StatusTag::Ignored),
            _   => None,
        }
    }

    /// Get the color associated with the status
    pub fn to_color(&self) -> Color {
        match self {
            StatusTag::Unmodified      => Color::White,
            StatusTag::Modified        => Color::Green,
            StatusTag::FileTypeChanged => Color::Yellow,
            StatusTag::Added           => Color::Yellow,
            StatusTag::Deleted         => Color::Red,
            StatusTag::Renamed         => Color::BrightBlue,
            StatusTag::Copied          => Color::BrightBlue,
            StatusTag::UpdatedUnmerged => Color::BrightBlue,

            StatusTag::Untracked       => Color::BrightCyan,
            StatusTag::Ignored         => Color::BrightCyan,
        }
    }
}

impl std::fmt::Display for StatusTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StatusTag::Unmodified      => " ",
            StatusTag::Modified        => "~",
            StatusTag::FileTypeChanged => "~",
            StatusTag::Added           => "+",
            StatusTag::Deleted         => "-",
            StatusTag::Renamed         => "~",
            StatusTag::Copied          => "~",
            StatusTag::UpdatedUnmerged => "~",

            StatusTag::Untracked       => "?",
            StatusTag::Ignored         => "!",
        })
    }
}

/// Does not derive from Ord because its ordering should not depend on its
/// status tag.
pub struct StatusEntry {
    pub status: StatusTag,
    pub path: String,
}

impl PartialEq for StatusEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for StatusEntry {}

impl PartialOrd for StatusEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for StatusEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

/// A data structure for `git status`'s output
///
/// Holds two sorted lists of `StatusEntry` structs; one for staged items and
/// one for unstaged changes.
#[derive(Default)]
pub struct Status {
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
    pub untracked: Vec<StatusEntry>,
}

/// Defines a few helpers to deal with the collection of entries as a whole
/// instead of the split into staged, unstaged, and untracked entries.
impl Status {
    fn new() -> Self {
        Default::default()
    }

    /// Retrieve an item at a unique index.
    ///
    /// The index is counted across staged, unstaged, and untracked items, in
    /// the priority order `staged -> unstaged -> untracked`. This assigns a
    /// unique index to each single entry.
    ///
    /// # Examples
    ///
    /// ```
    /// let status = parse_status("M  foo\n M bar").unwrap();
    ///
    /// assert_eq!(status.get(0).unwrap().path, PathBuf::from("foo"));
    /// assert_eq!(status.get(1).unwrap().path, PathBuf::from("bar"));
    /// assert_eq!(status.get(2), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&StatusEntry> {
        self.staged.iter().chain(&self.unstaged).chain(&self.untracked).nth(index)
    }

    /// Iterate over all entries (with no regards to their type) in sequence
    pub fn iter(&self) -> impl Iterator<Item = &StatusEntry> {
        self.staged.iter().chain(&self.unstaged).chain(&self.untracked)
    }
}

/// Runs the `git status` command and parses the output into a `Status`
/// structure.
///
/// Returns an error if the command fails or if the function is unable to parse
/// its output.
///
/// # Examples
///
/// ```
/// match status() {
///     Ok(status) {
///         for entry in &status.unstaged {
///             // ...
///         }
///     }
///     Err(err) {
///         // ...
///     }
/// }
/// ```
pub fn status() -> Result<Status, io::Error> {
    let output = Command::new("git")
                         .arg("status")
                         .arg("--porcelain")
                         .output();

    output
        .and_then(|output| {
            if !output.status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("`git status` failed with error: {}",
                            String::from_utf8(output.stderr).unwrap())
                ));
            }
            String::from_utf8(output.stdout)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
        })
        .and_then(|string| parse_status(&string))
        .and_then(|mut status| {
            status.staged.sort();
            status.unstaged.sort();
            Ok(status)
        })
}

/// Parse a UTF-8 text as a `git status` line.
///
/// On success, returns the entry and a flag that indicates whether the entry
/// is staged (true) or unstaged (false).
///
/// # Panics
///
/// May panic if the line is not in a valid `git status --porcelain` format.
fn parse_status_line(line: &str) -> (Option<StatusEntry>, Option<StatusEntry>) {
    let (tag_str, path_str) = line.split_at(2);

    // Create an entry object from a tag
    let make_entry = |tag: StatusTag| -> StatusEntry {
        StatusEntry{
            status: tag,
            path: String::from(path_str.trim())
        }
    };
    let is_modified = |tag: &StatusTag| tag != &StatusTag::Unmodified;

    return (
        tag_str.chars().nth(0).and_then(StatusTag::from).filter(is_modified).map(make_entry),
        tag_str.chars().nth(1).and_then(StatusTag::from).filter(is_modified).map(make_entry),
    );
}

/// Parse the output of `git status` into a `Status` struct.
fn parse_status(output: &str) -> Result<Status, io::Error> {
    let mut result: Status = Status::new();
    for line in output.lines().filter(|l| !l.is_empty()) {
        match parse_status_line(line) {
            (Some(staged), None)           => result.staged.push(staged),
            (None,         Some(unstaged)) => result.unstaged.push(unstaged),
            (Some(staged), Some(unstaged)) => {
                if staged.status != StatusTag::Untracked {
                    result.staged.push(staged);
                    result.unstaged.push(unstaged);
                }
                else {
                    result.untracked.push(staged);
                }
            }
            (None, None) => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Encountered malformed status line: {}", line)
            ))
        }
    }

    return Ok(result);
}
