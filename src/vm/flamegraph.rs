use std::fs::File;
use std::io::Write;
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Command, Stdio},
};

use super::function::Function;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Flamegraph {
    counts: HashMap<Vec<Function>, usize>,
}

#[allow(dead_code)]
impl Flamegraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, stack: Vec<Function>) {
        *self.counts.entry(stack).or_insert(0) += 1;
    }

    pub fn write(&self, path: &PathBuf) -> std::io::Result<()> {
        let mut child = Command::new("/Users/darren/Projects/FlameGraph/flamegraph.pl")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            let mut rows: Vec<(&Vec<Function>, &usize)> = self.counts.iter().collect();
            rows.sort_by(|a, b| b.1.cmp(a.1));
            for (stack, count) in rows {
                let stack_str = stack
                    .iter()
                    .map(|f| f.get_debug_name().to_owned())
                    .collect::<Vec<String>>()
                    .join(";");

                writeln!(stdin, "{} {}", stack_str, count)?;
            }
        } else {
            panic!("failed to open stdin");
        }

        let mut output_file = File::create(path)?;
        let mut child_stdout = child.stdout.take().expect("Failed to open child stdout");

        std::io::copy(&mut child_stdout, &mut output_file)?;

        let status = child.wait()?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "flamegraph.pl script failed",
            ));
        }

        Ok(())
    }
}
