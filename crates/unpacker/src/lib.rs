pub mod bundle;

use anyhow::Result;
use bundle::parse_header;
use std::fs::{File, OpenOptions, create_dir_all, read_dir};
use std::io::{BufReader, BufWriter, Write, copy};
use std::path::Path;
use zip::ZipArchive;

pub fn unpack(input: &str, output: &str) -> Result<()> {
    println!("Unpacking from {:?} to {:?}", input, output);

    let out_path = Path::new(output);
    create_dir_all(out_path)?;

    let log_path = out_path.join("unpacker.log");
    let logfile = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    let mut log = BufWriter::new(logfile);
    scan_dir(Path::new(input), output, &mut log)?;

    log.flush()?;
    println!("Done unpacking.");
    Ok(())
}

fn scan_dir(dir: &Path, output: &str, log: &mut impl Write) -> Result<()> {
    for entry in read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            scan_dir(&path, output, log)?;
        } else {
            match path.extension().and_then(|e| e.to_str()) {
                Some("ab") | Some("bin") => unpack_bundle(&path, log)?,
                Some("idx") => extract_zip(&path, output, Some(()))?,
                Some("usm") => extract_zip(&path, output, None)?,
                _ => writeln!(log, "SKIP [unknown]: {}", path.display())?,
            }
        }
    }

    Ok(())
}

fn unpack_bundle(path: &Path, log: &mut impl Write) -> Result<()> {
    let file = File::open(path)?;
    let mut zip = match ZipArchive::new(file) {
        Ok(z) => z,
        Err(e) => {
            writeln!(log, "SKIP [bad zip]: {}: {}", path.display(), e)?;
            return Ok(());
        }
    };

    let mut found_any = false;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.name().ends_with(".ab") || entry.name().ends_with(".bin") {
            found_any = true;
            println!("Found bundle: {:?}", entry.name());
            let mut reader = BufReader::new(&mut entry);
            let header = match parse_header(&mut reader) {
                Ok(h) => h,
                Err(e) => {
                    writeln!(log, "SKIP [bad header]: {}: {}", entry.name(), e)?;
                    continue;
                }
            };
            println!(
                "  format = {}; unity = {}",
                header.format_version, header.unity_version_built
            );
        } else {
            writeln!(
                log,
                "SKIP [not unity bundle]: {} in {}",
                entry.name(),
                path.display()
            )?;
        }
    }

    if !found_any {
        writeln!(log, "SKIP [no ab/bin]: {}", path.display())?;
    }

    Ok(())
}

fn extract_zip(path: &Path, output: &str, idx: Option<()>) -> Result<()> {
    let file = File::open(path)?;
    let mut zip = ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }

        let out_path = if idx.is_none() {
            Path::new(output).join(entry.name())
        } else {
            Path::new(output).join(path.file_name().unwrap())
        };

        if let Some(parent) = out_path.parent() {
            create_dir_all(parent)?;
        }

        let mut out_file = File::create(&out_path)?;
        copy(&mut entry, &mut out_file)?;
        println!("Extracted {:?}", out_path.display());
    }

    Ok(())
}
