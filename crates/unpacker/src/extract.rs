use anyhow::Result;
use std::fs::{File, create_dir_all};
use std::io::{Write, copy};
use std::path::Path;
use zip::ZipArchive;

use crate::scan_files;

pub fn all(input: &Path, output: &Path, log: &mut impl Write) -> Result<()> {
    for path in scan_files(input) {
        let path = path?;
        match path.extension().and_then(|e| e.to_str()) {
            Some("ab") | Some("bin") | Some("usm") => {
                extract_zip(&path, output, false)?;
            }
            Some("idx") => {
                extract_zip(&path, output, true)?;
            }
            _ => writeln!(log, "SKIP [unknown]: {}", path.display())?,
        }
    }

    Ok(())
}

fn extract_zip(path: &Path, output: &Path, idx: bool) -> Result<()> {
    let file = File::open(path)?;
    let mut zip = ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }

        let out_path = if idx {
            output.join(
                path.file_name()
                    .ok_or_else(|| anyhow::anyhow!("invalid idx path: {}", path.display()))?,
            )
        } else {
            output.join(entry.name())
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
