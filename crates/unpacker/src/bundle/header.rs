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

impl BundleHeader {
    pub fn parse(reader: &mut impl BufRead) -> Result<Self> {
        let signature = super::read_cstring(reader)?;
        if signature != "UnityFS" {
            bail!("Not a UnityFS bundle, got '{}'", signature);
        }

        Ok(Self {
            format_version: super::read_u32(reader)?,
            unity_version_min: super::read_cstring(reader)?,
            unity_version_built: super::read_cstring(reader)?,
            file_size: super::read_i64(reader)?,
            compressed_blocks_size: super::read_u32(reader)?,
            decompressed_blocks_size: super::read_u32(reader)?,
            flags: super::read_u32(reader)?,
        })
    }
}
