use anyhow::Result;
use std::io::{BufRead, Seek, SeekFrom};

use super::common::PPtr;
use crate::bundle::read::*;

pub struct MonoBehaviour {
    pub name: String,
    pub data: Vec<u8>,
}

impl MonoBehaviour {
    pub fn parse(
        reader: &mut (impl BufRead + Seek),
        abs_offset: u64,
        byte_size: u32,
    ) -> Result<Self> {
        reader.seek(SeekFrom::Start(abs_offset))?;
        let _game_object = PPtr::parse(reader)?;
        let _enabled = byte(reader)?;
        align4(reader, 0)?;
        let _script = PPtr::parse(reader)?;

        let name = String::from_utf8(aligned_bytes(reader)?)?;
        let data_start = reader.stream_position()?;
        let data_end = abs_offset + byte_size as u64;
        let data_size = data_end - data_start;
        let mut data = vec![0u8; data_size as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { name, data })
    }
}
