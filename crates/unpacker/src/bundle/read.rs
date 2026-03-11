use anyhow::Result;
use std::io::BufRead;

pub fn cstring(reader: &mut impl BufRead) -> Result<String> {
    let mut buf = Vec::new();
    reader.read_until(0, &mut buf)?;
    buf.pop();
    Ok(String::from_utf8(buf)?)
}

pub fn u16_be(reader: &mut impl BufRead) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn u16_le(reader: &mut impl BufRead) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

pub fn u32_be(reader: &mut impl BufRead) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub fn u32_le(reader: &mut impl BufRead) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn u64_be(reader: &mut impl BufRead) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

pub fn u64_le(reader: &mut impl BufRead) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn i64_be(reader: &mut impl BufRead) -> Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_be_bytes(buf))
}

pub fn i64_le(reader: &mut impl BufRead) -> Result<i64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(i64::from_le_bytes(buf))
}
