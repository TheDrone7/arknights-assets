use anyhow::{Result, bail};
use std::io::BufRead;

pub struct BundleHeader {
    pub format_version: u32,
    pub unity_version_min: String,
    pub unity_version_built: String,
    pub file_size: i64,
    pub compressed_blocks_size: u32,
    pub decompressed_blocks_size: u32,
    pub flags: u32,
}

pub fn parse_header(reader: &mut impl BufRead) -> Result<BundleHeader> {
    let signature = read_cstring(reader)?;
    if signature != "UnityFS" {
        bail!("Not a UnityFS bundle, got '{:?}'", signature);
    }

    Ok(BundleHeader {
        format_version: read_u32(reader)?,
        unity_version_min: read_cstring(reader)?,
        unity_version_built: read_cstring(reader)?,
        file_size: read_i64(reader)?,
        compressed_blocks_size: read_u32(reader)?,
        decompressed_blocks_size: read_u32(reader)?,
        flags: read_u32(reader)?,
    })
}

fn read_cstring(reader: &mut impl BufRead) -> Result<String> {
    let mut buf = Vec::new();
    reader.read_until(0, &mut buf)?;
    buf.pop();
    Ok(String::from_utf8(buf)?)
}

fn read_u32(reader: &mut impl BufRead) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_i64(reader: &mut impl BufRead) -> Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}
