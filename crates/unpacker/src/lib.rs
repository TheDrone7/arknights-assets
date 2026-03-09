use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn unpack(input: &str, output: &str) -> Result<()> {
    println!("Unpacking from {:?} to {:?}", input, output);
    scan_dir(Path::new(input))?;

    Ok(())
}

fn scan_dir(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            scan_dir(&path)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("ab") {
            println!("Found: {}", path.display());
        }
    }

    Ok(())
}
