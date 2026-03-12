pub mod bundle;
mod extract;
mod lz4inv;

use anyhow::Result;
use bundle::UnityBundle;

use std::fs::{OpenOptions, create_dir_all, read_dir, remove_dir_all};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn unpack(input: &str, output: &str) -> Result<()> {
    println!("Unpacking from {:?} to {:?}", input, output);

    let out_path = Path::new(output);
    let tmp_path = out_path.join(".tmp");
    create_dir_all(out_path)?;
    create_dir_all(&tmp_path)?;

    let log_path = out_path.join("unpacker.log");
    let logfile = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    let mut log = BufWriter::new(logfile);

    println!("[1/3] Extracting...");
    extract::all(Path::new(input), &tmp_path, &mut log)?;

    println!("[2/3] Parsing unity bundles...");
    parse_all(&tmp_path, out_path, &mut log)?;

    remove_dir_all(&tmp_path)?;
    log.flush()?;

    println!("Done unpacking.");
    Ok(())
}

fn scan_files(root: &Path) -> impl Iterator<Item = Result<PathBuf>> {
    let mut stack = vec![root.to_path_buf()];

    std::iter::from_fn(move || {
        while let Some(path) = stack.pop() {
            if path.is_dir() {
                match read_dir(&path) {
                    Ok(entries) => {
                        for entry in entries.flatten() {
                            stack.push(entry.path());
                        }
                    }
                    Err(e) => return Some(Err(e.into())),
                }
            } else {
                return Some(Ok(path));
            }
        }

        None
    })
}

fn parse_all(input: &Path, output: &Path, log: &mut impl Write) -> Result<()> {
    for path in scan_files(input) {
        let path = path?;
        match path.extension().and_then(|e| e.to_str()) {
            Some("ab") | Some("bin") => parse_bundle(&path, output, log)?,
            _ => continue,
        }
    }

    Ok(())
}

fn parse_bundle(path: &Path, out_root: &Path, log: &mut impl Write) -> Result<()> {
    let tmp_root = out_root.join(".tmp");
    let relative = path
        .parent()
        .unwrap_or(&tmp_root)
        .strip_prefix(&tmp_root)
        .unwrap_or(Path::new(""));
    let out_base = out_root.join(relative);
    if let Err(e) = UnityBundle::process(path, &out_base) {
        writeln!(log, "{}", e)?;
        return Ok(());
    }
    Ok(())
}
