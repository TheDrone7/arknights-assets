pub mod bundle;
mod extract;
mod lz4inv;

use anyhow::Result;
use std::fs::{File, OpenOptions, create_dir_all, read_dir, remove_dir_all};
use std::io::{BufReader, BufWriter, Write};
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
    parse_all(&tmp_path, &mut log)?;

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

fn parse_all(input: &Path, log: &mut impl Write) -> Result<()> {
    for path in scan_files(input) {
        let path = path?;
        match path.extension().and_then(|e| e.to_str()) {
            Some("ab") | Some("bin") => parse_bundle(&path, log)?,
            _ => continue,
        }
    }

    Ok(())
}

fn parse_bundle(path: &Path, log: &mut impl Write) -> Result<()> {
    let mut reader = BufReader::new(File::open(path)?);
    let bundle = match bundle::UnityBundle::parse(&mut reader) {
        Ok(b) => b,
        Err(e) => {
            writeln!(log, "SKIP [bad asset bundle] | [{}]: {}", path.display(), e)?;
            return Ok(());
        }
    };
    println!(
        "File: {}\n  blocks={}; nodes={}",
        path.display(),
        bundle.info.blocks.len(),
        bundle.info.nodes.len()
    );

    let dec_path = path.with_extension("dec");
    let dec_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&dec_path)?;
    let mut dec_writer = BufWriter::new(dec_file);
    let bytes_size = match bundle.decompress(&mut reader, &mut dec_writer) {
        Ok(size) => size,
        Err(e) => {
            writeln!(log, "SKIP [bad compression] | [{}]: {}", path.display(), e)?;
            return Err(e);
        }
    };
    dec_writer.flush()?;
    println!("  dec: {} (size: {}B)", dec_path.display(), bytes_size);

    let mut dec_reader = BufReader::new(File::open(&dec_path)?);
    for sf in bundle.get_serialized(&mut dec_reader)? {
        println!(
            "  - {}:: v{}; {} objects",
            sf.name,
            sf.version,
            sf.objects.len()
        );

        for obj in sf.objects {
            println!(
                "    - path: {}; class: {}, size: {}B",
                obj.path_id, obj.class_id, obj.byte_size
            );
        }
    }

    println!("\n");

    Ok(())
}
