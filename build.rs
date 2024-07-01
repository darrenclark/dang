use std::fs::{self};
use std::io;
use std::path::Path;
use std::time::SystemTime;

fn main() {
    println!("cargo::rerun-if-changed=stdlib/");
    println!(
        "cargo::rustc-env=DANG_STDLIB_LAST_MODIFIED={:?}",
        latest_mtime(Path::new("stdlib/")).unwrap()
    );
}

fn latest_mtime(dir: &Path) -> io::Result<SystemTime> {
    let mut latest = SystemTime::UNIX_EPOCH;

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let mtime: SystemTime = if path.is_dir() {
                latest_mtime(&path)?
            } else {
                entry.metadata()?.modified()?
            };
            if mtime > latest {
                latest = mtime;
            }
        }
    }
    Ok(latest)
}
