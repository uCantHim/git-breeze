use clap::{Parser, Subcommand};

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
    }
}

fn main() {
    let prog = Program::parse();
    match prog.command {
        Command::Status => {
            match cmd::status() {
                Ok(status) => {
                    print_status(status);
                }
                Err(err) => {
                    println!("An error occured: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Command::Commit{ git_args } => {
            let output = std::process::Command::new("git")
                .arg("commit")
                .args(git_args)
                .spawn();

            exit(match output {
                Ok(child) => child.wait_with_output(),
                Err(err) => Err(err)
            });
        }
    }
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
    let print = |entry, i| println!(" [{}]   {}", i, entry);

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
}
