//! Invoke and parse the output of the `git branch` command.

use std::io;
use std::process::Command;

#[derive(Default)]
pub struct Branch {
    pub branches: Vec<String>,

    /// The n-th entry in `commits` is the commit that the n-th entry in
    /// `branches` has currently checked out. Is a string "<hash> <message>".
    pub commits: Vec<String>,

    /// `branches.get(current)` is the currently checked out branch
    pub current: usize,
}

impl Branch {
    fn new() -> Self {
        Default::default()
    }
}

/// Invoke `git branch` and parse its output into a `Branch` structure.
pub fn branch() -> io::Result<Branch> {
    let output = Command::new("git").arg("branch").arg("-v").output()?;

    if output.status.success() {
        parse_branch(String::from_utf8(output.stdout).expect("`git branch` output must be unicode"))
    }
    else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("`git status` failed with error: {}",
                    String::from_utf8(output.stderr).unwrap())
        ))
    }
}

fn parse_branch(output: String) -> io::Result<Branch> {
    let mut result = Branch::new();
    for line in output.lines().filter(|l| !l.is_empty()) {
        let (prefix, branch) = line.split_at(2);
        let (branch, commit) = branch.split_once(' ').expect("unexpected `git branch -v` format");

        if prefix.starts_with('*') {
            result.current = result.branches.len();
        }
        result.branches.push(String::from(branch));
        result.commits.push(String::from(commit));
    }

    Ok(result)
}
