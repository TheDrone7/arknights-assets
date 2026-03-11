use anyhow::Result;
use std::io::{BufRead, Seek, SeekFrom};

pub struct SerializedFile {
    pub name: String,
    pub metadata_size: u32,
    pub file_size: i64,
    pub version: u32,
    pub data_offset: i64,
    pub endianness: u8,
}

impl SerializedFile {
    pub fn parse(
        name: &str,
        reader: &mut (impl BufRead + Seek),
        offset: u64,
        _size: u64,
    ) -> Result<Self> {
        let name = name.to_string();

        reader.seek(SeekFrom::Start(offset))?;
        let metadata_size = super::read::u32_be(reader)?;
        let file_size = super::read::u32_be(reader)? as i64;
        let version = super::read::u32_be(reader)?;
        let data_offset = super::read::u32_be(reader)? as i64;
        if version < 9 {
            return Ok(Self {
                name,
                metadata_size,
                file_size,
                version,
                data_offset,
                endianness: 0,
            });
        }

        let endianness = read_endian(reader)?;
        if version < 22 {
            return Ok(Self {
                name,
                metadata_size,
                file_size,
                version,
                data_offset,
                endianness,
            });
        }

        let metadata_size = super::read::u32_be(reader)?;
        let file_size = super::read::i64_be(reader)?;
        let data_offset = super::read::i64_be(reader)?;
        reader.seek(SeekFrom::Current(8))?;

        Ok(Self {
            name,
            metadata_size,
            file_size,
            version,
            data_offset,
            endianness,
        })
    }
}

fn read_endian(reader: &mut impl BufRead) -> Result<u8> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}
