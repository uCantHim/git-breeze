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
    Commit
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
                }
            }
        }
        Command::Commit => {

        }
    }
}

/// Print contents of a `cmd::Status` struct
fn print_status(status: cmd::Status) {
    if !status.staged.is_empty() {
        println!("\n--- Staged Items ---\n");
        for entry in &status.staged {
            println!("{}", entry);
        }
    }
    if !status.unstaged.is_empty() {
        println!("\n--- Unstaged Items ---\n");
        for entry in &status.unstaged {
            println!("{}", entry);
        }
    }
}
