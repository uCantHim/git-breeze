use clap::{Parser, Subcommand};
use colored::*;

pub mod cmd;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Program {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Status,
    Commit {
        #[arg(last=true)]
        git_args: Vec<String>,
    },
    Add {
        files: Vec<String>,

        #[arg(last=true)]
        git_args: Vec<String>,
    },
    Diff {
        files: Vec<String>,

        #[arg(last=true)]
        git_args: Vec<String>,
    },
    Reset {
        files: Vec<String>,

        #[arg(last=true)]
        git_args: Vec<String>,
    },
    Checkout {
        files: Vec<String>,

        #[arg(last=true)]
        git_args: Vec<String>,
    },
    Stash {
        files: Vec<String>,

        #[arg(last=true)]
        git_args: Vec<String>,
    },
    Branch {
        #[arg(help="Provide a name to create a new branch")]
        name: Option<String>,

        #[arg(last=true)]
        git_args: Vec<String>,
    }
}

fn main() {
    fn replace_then_run(git_cmd: &str, args: Vec<String>, mut git_args: Vec<String>) {
        // Always separate the file arguments from the git arguments because
        // we can.
        git_args.push(String::from("--"));

        // Replace number arguments with entries from `git status`, then run
        // the git command.
        match cmd::status().and_then(|status| cmd::replace_number_args(args, &status))
        {
            Ok(args) => run_git(git_cmd, &git_args, &args),
            Err(err) => exit_with_error(err),
        }
    }

    let prog = Program::parse();
    match prog.command {
        Command::Status => {
            match cmd::status() {
                Ok(status) => print_status(status),
                Err(err) => exit_with_error(err),
            }
        }
        Command::Commit { git_args } => {
            let output = std::process::Command::new("git")
                .arg("commit")
                .args(git_args)
                .spawn();

            exit(match output {
                Ok(child) => child.wait_with_output(),
                Err(err) => Err(err)
            });
        }
        Command::Add { files, git_args } => {
            replace_then_run("add", files, git_args);
        }
        Command::Diff { files, git_args } => {
            replace_then_run("diff", files, git_args);
        }
        Command::Reset { files, git_args } => {
            replace_then_run("reset", files, git_args);
        }
        Command::Checkout { files, git_args } => {
            replace_then_run("checkout", files, git_args);
        }
        Command::Stash { files, git_args } => {
            replace_then_run("stash", files, git_args);
        }
        Command::Branch { name: Some(name), git_args } => {
            run_git("branch", &vec![name], &git_args);
        }
        Command::Branch { name: None, git_args: _ } => {
            match cmd::branch() {
                Ok(branch) => print_branch(branch),
                Err(err)   => exit_with_error(err),
            }
        }
    }
}

pub fn run_git(git_cmd: &str, args: &Vec<String>, git_args: &Vec<String>) {
    let output = std::process::Command::new("git")
        .arg(git_cmd)
        .args(args)
        .args(git_args)
        .spawn();

    exit(match output {
        Ok(child) => child.wait_with_output(),
        Err(err)  => Err(err)
    });
}

fn exit_with_error<E: std::fmt::Display>(err: E) {
    println!("An error occured: {}", err);
    std::process::exit(1);
}

fn exit(output: std::io::Result<std::process::Output>) {
    match output {
        Ok(output) => {
            if !output.status.success() {
                println!("{}", String::from_utf8(output.stderr).unwrap_or(String::new()));
            }
            std::process::exit(output.status.code().unwrap_or(1));
        },
        Err(err) => {
            println!("An error occured: {}", err);
            std::process::exit(1);
        },
    };
}

/// Print contents of a `cmd::Status` struct
fn print_status(status: cmd::Status) {
    let print = |entry: &cmd::StatusEntry, i: u32| {
        let color = entry.status.to_color();
        let row_color = match i % 2 == 0 {
            true  => Color::White,
            false => Color::TrueColor{ r: 140, g: 140, b: 140 }
        };
        println!(
            " {}   {} {}",
            entry.status.to_string().color(color),
            format!("[{}]", i).color(row_color),
            entry.path.color(color))
    };

    let mut i = 0;
    if !status.staged.is_empty() {
        println!("\n--- Staged Items ---\n");
        for entry in &status.staged {
            print(entry, i);
            i += 1;
        }
    }
    if !status.unstaged.is_empty() {
        println!("\n--- Unstaged Items ---\n");
        for entry in &status.unstaged {
            print(entry, i);
            i += 1;
        }
    }
    if !status.untracked.is_empty() {
        println!("\n--- Untracked Items ---\n");
        for entry in &status.untracked {
            print(entry, i);
            i += 1;
        }
    }

    println!("");
}

/// Print contents of a `cmd::Branch` struct
fn print_branch(info: cmd::Branch) {
    let it = info.branches.iter().zip(info.commits.iter()).enumerate();
    for (i, (branch, _)) in it {
        let prefix       = if i == info.current { " * " } else { "   " };
        let branch_color = if i == info.current { Color::Magenta } else { Color::Green };
        let row_color = match i % 2 == 0 {
            true  => Color::White,
            false => Color::TrueColor{ r: 140, g: 140, b: 140 }
        };

        println!(
            "{}  [{}] {}",
            prefix,
            format!("{}", i).color(row_color),
            branch.color(branch_color)
        );
    }
}
