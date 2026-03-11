use anyhow::Result;
use std::io::{BufRead, Seek, SeekFrom};

pub fn cstring(reader: &mut impl BufRead) -> Result<String> {
    let mut buf = Vec::new();
    reader.read_until(0, &mut buf)?;
    buf.pop();
    Ok(String::from_utf8(buf)?)
}

pub fn byte(reader: &mut impl BufRead) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
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

pub fn i16_be(reader: &mut impl BufRead) -> Result<i16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

pub fn i16_le(reader: &mut impl BufRead) -> Result<i16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

pub fn i32_be(reader: &mut impl BufRead) -> Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

pub fn i32_le(reader: &mut impl BufRead) -> Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
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

pub fn f32_le(reader: &mut impl BufRead) -> Result<f32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

pub fn align4(reader: &mut (impl BufRead + Seek), base: u64) -> Result<()> {
    let pos = reader.stream_position()? - base;
    let aligned = (pos + 3) & !3;
    if aligned > pos {
        reader.seek(SeekFrom::Current((aligned - pos) as i64))?;
    }

    Ok(())
}

pub fn aligned_bytes(reader: &mut (impl BufRead + Seek)) -> Result<Vec<u8>> {
    let len = u32_le(reader)? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    align4(reader, 0)?;

    Ok(buf)
}
