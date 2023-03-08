use crate::cmd;

use colored::*;

impl std::fmt::Display for cmd::Status {
    /// Print contents of a `cmd::Status` struct
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let print = |f: &mut std::fmt::Formatter, entry: &cmd::StatusEntry, i: u32| {
            let color = entry.status.to_color();
            let row_color = match i % 2 == 0 {
                true  => Color::White,
                false => Color::TrueColor{ r: 140, g: 140, b: 140 }
            };
            writeln!(f,
                " {}   {} {}",
                entry.status.to_string().color(color),
                format!("[{}]", i).color(row_color),
                entry.path.color(color))
        };

        let mut i = 0;
        if !self.staged.is_empty() {
            if i > 0 { writeln!(f, "")?; }
            writeln!(f, "--- Staged Items ---\n")?;
            for entry in &self.staged {
                print(f, entry, i)?;
                i += 1;
            }
        }
        if !self.unstaged.is_empty() {
            if i > 0 { writeln!(f, "")?; }
            writeln!(f, "--- Modified Items ---\n")?;
            for entry in &self.unstaged {
                print(f, entry, i)?;
                i += 1;
            }
        }
        if !self.untracked.is_empty() {
            if i > 0 { writeln!(f, "")?; }
            writeln!(f, "--- Untracked Items ---\n")?;
            for entry in &self.untracked {
                print(f, entry, i)?;
                i += 1;
            }
        }

        if i == 0 {
            write!(f, "Working tree clean.")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for cmd::Branch {
    /// Print contents of a `cmd::Branch` struct
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let it = self.branches.iter().zip(self.commits.iter()).enumerate();
        for (i, (branch, _)) in it {
            let prefix       = if i == self.current { " * " } else { "   " };
            let branch_color = if i == self.current { Color::Magenta } else { Color::Green };
            let row_color = match i % 2 == 0 {
                true  => Color::White,
                false => Color::TrueColor{ r: 140, g: 140, b: 140 }
            };

            writeln!(f,
                "{}  [{}] {}",
                prefix,
                format!("{}", i).color(row_color),
                branch.color(branch_color)
            )?;
        }

        Ok(())
    }
}
