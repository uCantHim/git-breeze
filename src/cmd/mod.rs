mod status;
mod branch;
mod display;

pub use status::status;
pub use status::Status;
pub use status::StatusEntry;

pub use branch::branch;
pub use branch::Branch;

use std::io;
use std::path::PathBuf;

/// May fail (return an Err-result) if a number with no corresponding entry
/// in the status is encountered, or a number range is invalid.
pub fn replace_number_args(args: Vec<String>, status: &Status) -> io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();

    // Push a status entry if the index exists in the Status structure
    let try_push = |i, result: &mut Vec<String>| -> io::Result<()> {
        let entry = status.get(i).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("No status entry with number {} exists.", i))
        )?;
        result.push(entry.path.clone());

        return Ok(());
    };

    for arg in args {
        // Try to parse the argument as a number range
        let split = arg.split_once('-')
            .map(|(a, b)| (a.parse::<usize>(), b.parse::<usize>()))
            .and_then(|(a, b)| Some((a.ok()?, b.ok()?)));

        match split {
            Some((begin, end)) => {
                if begin > end {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                              format!("Invalid range [{}, {}]", begin, end)));
                }
                for i in begin .. end + 1 {
                    try_push(i, &mut result)?;
                }
            }
            None => {
                // Try to parse the argument as a single number
                match arg.parse::<usize>() {
                    Ok(i) => try_push(i, &mut result)?,
                    // Unable to parse the argument as a number or a range.
                    // Add it to the result unaltered.
                    _     => result.push(arg),
                }
            }
        }
    }

    Ok(result)
}

fn get_git_root() -> io::Result<PathBuf> {
    std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .map(|output| String::from_utf8(output.stdout).expect(""))
        .map(|string| PathBuf::from(string.trim()))
}

pub fn make_relative_to_git_root(mut files: Vec<String>) -> io::Result<Vec<String>> {
    let git_root = get_git_root()?;
    for file in &mut files {
        *file = git_root.join(&file).to_str().unwrap().to_string();
    }

    Ok(files)
}
