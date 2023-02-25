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
            cmd::status();
        }
        Command::Commit => {
            cmd::commit();
        }
    }
}
