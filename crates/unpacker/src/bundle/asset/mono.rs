use anyhow::Result;
use std::fs::{create_dir_all, write};
use std::io::{BufRead, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::common::PPtr;
use crate::bundle::read::*;
use crate::bundle::serialized::TypeTreeNode;
use crate::bundle::typetree;

#[derive(Debug)]
pub struct MonoBehaviour {
    pub name: String,
    pub data: serde_json::Value,
}

impl MonoBehaviour {
    pub fn parse(
        nodes: &[TypeTreeNode],
        reader: &mut (impl BufRead + Seek),
        abs_offset: u64,
        byte_size: u32,
    ) -> Result<Self> {
        reader.seek(SeekFrom::Start(abs_offset))?;

        if nodes.is_empty() {
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
            return Ok(Self {
                name,
                data: serde_json::Value::Null,
            });
        }

        let data = typetree::decode(nodes, reader)?;
        let name = data["m_Name"].as_str().unwrap_or("").to_string();
        Ok(Self { name, data })
    }

    pub fn extract(&self, dir: &Path, id: usize) -> Result<PathBuf> {
        create_dir_all(dir)?;
        let mono_name = if self.name.is_empty() {
            &format!("mono{}", id)
        } else {
            &self.name
        };
        let out_path = dir.join(mono_name);
        write(&out_path, serde_json::to_vec(&self.data)?)?;

        Ok(out_path)
    }
}
