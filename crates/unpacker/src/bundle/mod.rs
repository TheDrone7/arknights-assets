mod block;
mod header;

use anyhow::Result;
use std::io::{BufRead, Seek, Write};

use block::BlockInfo;
use header::BundleHeader;

pub struct UnityBundle {
    pub header: BundleHeader,
    pub info: BlockInfo,
}

impl UnityBundle {
    pub fn parse(reader: &mut (impl BufRead + Seek)) -> Result<Self> {
        let header = BundleHeader::parse(reader)?;
        let info = BlockInfo::parse(reader, &header)?;

        Ok(Self { header, info })
    }

    pub fn decompress(
        &self,
        reader: &mut (impl BufRead + Seek),
        output: &mut impl Write,
    ) -> Result<usize> {
        self.info.decompress(reader, output, &self.header)
    }
}

fn read_cstring(reader: &mut impl BufRead) -> Result<String> {
    let mut buf = Vec::new();
    reader.read_until(0, &mut buf)?;
    buf.pop();
    Ok(String::from_utf8(buf)?)
}

fn read_u16(reader: &mut impl BufRead) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32(reader: &mut impl BufRead) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u64(reader: &mut impl BufRead) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

fn read_i64(reader: &mut impl BufRead) -> Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}
