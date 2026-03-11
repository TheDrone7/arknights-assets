use anyhow::Result;
use std::io::BufRead;

use crate::bundle::read::*;

#[derive(Debug)]
pub struct Rectangle(pub [f32; 4]);

impl Rectangle {
    pub fn parse(reader: &mut impl BufRead) -> Result<Self> {
        let rect = [
            f32_le(reader)?,
            f32_le(reader)?,
            f32_le(reader)?,
            f32_le(reader)?,
        ];

        Ok(Self(rect))
    }
}

#[derive(Debug)]
pub struct PPtr {
    pub file_id: i32,
    pub path_id: i64,
}

impl PPtr {
    pub fn parse(reader: &mut impl BufRead) -> Result<Self> {
        let file_id = i32_le(reader)?;
        let path_id = i64_le(reader)?;
        Ok(Self { file_id, path_id })
    }
}
