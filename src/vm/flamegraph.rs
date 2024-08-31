use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Command, Stdio},
};

use super::function::Function;

static FLAMEGRAPH_PL: &str = include_str!("../../vendor/flamegraph.pl");
static FLAMEGRAPH_PL_PATH: &str = "/tmp/dang-flamegraph.pl";

#[derive(Debug)]
#[allow(dead_code)]
pub struct Flamegraph {
    counts: HashMap<Vec<Function>, usize>,
    interval: Duration,
    last: Instant,
}

#[allow(dead_code)]
impl Flamegraph {
    pub fn new(interval: Duration) -> Self {
        Self {
            counts: HashMap::new(),
            interval,
            last: Instant::now(),
        }
    }

    pub fn check_if_time_for_measurement(&mut self) -> bool {
        let now = Instant::now();
        let time_since_last = now.duration_since(self.last);
        if time_since_last > self.interval {
            self.last = now;
            true
        } else {
            false
        }
    }

    pub fn increment(&mut self, stack: Vec<Function>) {
        *self.counts.entry(stack).or_insert(0) += 1;
    }

    pub fn write(&self, path: &PathBuf) -> std::io::Result<()> {
        Self::create_flamegraph_pl().expect("Failed to write /tmp/dang-flamegraph.pl");

        let mut child = Command::new(FLAMEGRAPH_PL_PATH)
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

    fn create_flamegraph_pl() -> std::io::Result<()> {
        let mut open_opts = OpenOptions::new();
        open_opts.write(true).create(true).truncate(true);
        if cfg!(unix) {
            open_opts.mode(0o755);
        }

        let mut file = open_opts.open(FLAMEGRAPH_PL_PATH)?;
        file.write_all(FLAMEGRAPH_PL.as_bytes())?;
        Ok(())
    }
}
