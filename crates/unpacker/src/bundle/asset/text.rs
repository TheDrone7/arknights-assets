use anyhow::Result;
use std::io::{BufRead, Seek, SeekFrom};

use crate::bundle::read::*;

pub struct TextAsset {
    pub name: String,
    pub data: Vec<u8>,
}

impl TextAsset {
    pub fn parse(reader: &mut (impl BufRead + Seek), abs_offset: u64) -> Result<Self> {
        reader.seek(SeekFrom::Start(abs_offset))?;
        let name_bytes = aligned_bytes(reader)?;
        let name = String::from_utf8(name_bytes)?;
        let data = aligned_bytes(reader)?;

        Ok(TextAsset { name, data })
    }
}
