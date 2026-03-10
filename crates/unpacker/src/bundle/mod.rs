mod block;
mod header;

use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use block::BlockInfo;
use header::BundleHeader;

pub struct UnityBundle {
    pub header: BundleHeader,
    pub info: BlockInfo,
}

impl UnityBundle {
    pub fn parse(path: &Path, log: &mut impl Write) -> Result<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let header = match BundleHeader::parse(&mut reader) {
            Ok(h) => h,
            Err(e) => {
                writeln!(log, "SKIP [bad header] | [{}]: {}", path.display(), e)?;
                return Err(e);
            }
        };
        let info = match BlockInfo::parse(&mut reader, &header) {
            Ok(i) => i,
            Err(e) => {
                writeln!(log, "SKIP [bad block info] | [{}]: {}", path.display(), e)?;
                return Err(e);
            }
        };

        Ok(Self { header, info })
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
