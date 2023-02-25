use std::io;
use std::path::PathBuf;
use std::process::Command;

enum StatusEntryType {
    Modified,
    Untracked,
}

struct StatusEntry (StatusEntryType, PathBuf);

struct Status {
    entries: Vec<StatusEntry>,
}

pub fn status() {
    let output = Command::new("git")
                         .arg("status")
                         .arg("--porcelain")
                         .output();

    // let output = Command::new("git")
    //         .arg("status")
    //         .arg("--porcelain")
    //         .output()
    //         .and_then(|output| Ok(String::from_utf8(output.stdout).expect("")))
    //         .and_then(|string| parse_status(&string));

    let foo = output
        .and_then(|output| {
            match String::from_utf8(output.stdout) {
                Ok(val) => Ok(val),
                Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
            }
        })
        .and_then(|string| parse_status(&string));

    match output {
        Ok(out) => {
            let output = String::from_utf8(out.stdout)
                                .expect("git status did not output UTF-8 text!");
            parse_status(&output);
        }
        Err(err) => {
            println!("An error occured: {}", err);
        }
    }
}

fn parse_status_line(line: &str) -> Result<StatusEntry, io::Error> {
    let line: Vec<&str> = line.trim().splitn(2, ' ').collect();
    let tag = line.get(0).expect("git status line must have a tag");
    let file = line.get(1).expect("git status line must have a file name");
    match tag {
        &"M"  => Ok(StatusEntryType::Modified),
        &"??" => Ok(StatusEntryType::Untracked),
        _     => Err(io::Error::new(io::ErrorKind::InvalidData,
                                    format!("Unknown `git status` tag \"{}\"", tag)))
    }.and_then(|t| Ok(StatusEntry(t, PathBuf::from(file))))
}

fn parse_status(output: &str) -> Result<Status, io::Error> {
    let mut result: Status = Status{ entries: Vec::new() };
    for line in output.split("\n") {
        match parse_status_line(line) {
            Ok(entry) => result.entries.push(entry),
            Err(err)  => return Err(err)
        }
    }

    return Ok(result);
}

pub fn commit() {
    println!("Called git commit!");
}
