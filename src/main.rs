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
    /// Improved `git checkout`.
    ///
    /// Calling this with a single argument treats the argument as a branch to
    /// check out. If you want to revert changes to a single file by its
    /// numbered status index, call it with a separator:
    ///
    ///  $ git-breeze checkout -- 0
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
    let prog = Program::parse();
    match prog.command {
        Command::Status => {
            match cmd::status() {
                Ok(status) => println!("{}", status),
                Err(err) => exit_with_error(err),
            }
        }
        Command::Commit { git_args } => {
            run_git("commit", &git_args, &Vec::new());
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
            checkout(files, git_args);
        }
        Command::Stash { files, git_args } => {
            replace_then_run("stash", files, git_args);
        }
        Command::Branch { name: Some(name), git_args } => {
            run_git("branch", &vec![name], &git_args);
        }
        Command::Branch { name: None, git_args: _ } => {
            match cmd::branch() {
                Ok(branch) => println!("{}", branch),
                Err(err)   => exit_with_error(err),
            }
        }
    }
}

/// Execute a checkout command
///
/// Performs some more complicated logic to detect the difference between an
/// intended branch change and reverting changes to a single file.
fn checkout(files: Vec<String>, git_args: Vec<String>) {
    match files[..] {
        [] => {
            let args = cmd::status().and_then(|status| cmd::replace_number_args(git_args, &status));
            match args {
                Ok(args) => run_git("checkout", &args, &Vec::new()),
                Err(err) => exit_with_error(err),
            }
        },
        // If only a single argument is provided, try to treat it as a
        // branch name
        [ref single] => {
            let replaced = single.parse::<usize>().ok()
                .and_then(|num| {
                    let branch = cmd::branch().ok()?;
                    branch.branches.get(num).map(|b| b.clone())
                });
            match replaced {
                Some(branch) => run_git("checkout", &git_args, &vec![branch]),
                None => replace_then_run("checkout", files, git_args),
            }
        },
        // Otherwise, treat args as file names and run default replacement
        _ => replace_then_run("checkout", files, git_args),
    }
}

/// Replace numbers in file arguments with the corresponding status entry, then
/// run the specified git command with `git_args` and `args` (file names).
///
/// Explicitly treats `args` as file names to the respective git command by
/// separating them from `git_args` with `--`.
///
/// To execute commands without any of these semantics, use the `run_git`
/// function.
///
/// # Example
///
/// If the working tree has modifications to the file `foo.txt`, the code
///
/// ```
/// let args = vec!["0"];
/// replace_then_run("add", args, vec!["-n"]);
/// ```
///
/// executes the git command
///
/// ```bash
/// git add -n -- foo.txt
/// ```
///
/// which would output something like "add 'foo.txt'".
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

pub fn run_git(git_cmd: &str, git_args: &Vec<String>, args: &Vec<String>) {
    let output = std::process::Command::new("git")
        .arg(git_cmd)
        .args(git_args)
        .args(args)
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
