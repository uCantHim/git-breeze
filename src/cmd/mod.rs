mod status;

pub use status::status;
pub use status::Status;
pub use status::StatusEntry;

use std::io;

pub fn replace_number_args(args: Vec<String>, status: &Status) -> io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();
    for arg in args {
        match arg.parse::<usize>() {
            Ok(i) => match status.get(i).map(|entry| entry.path.clone()) {
                Some(path) => {
                    result.push(path);
                },
                None => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, ""));
                }
            }
            _ => result.push(arg),
        }
    }

    Ok(result)
}
