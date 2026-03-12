use anyhow::{Result, anyhow};
use std::io::{BufRead, Seek, SeekFrom};

use super::read::*;

pub struct ObjectData {
    pub unity_version: (u32, u32),
    pub path_id: i64,
    pub byte_start: i64,
    pub byte_size: u32,
    pub class_id: i32,
}

pub struct SerializedFile {
    pub name: String,
    pub node_offset: u64,
    pub metadata_size: u32,
    pub file_size: i64,
    pub version: u32,
    pub data_offset: i64,
    pub endianness: u8,
    pub objects: Vec<ObjectData>,
}

impl SerializedFile {
    pub fn parse(
        name: &str,
        reader: &mut (impl BufRead + Seek),
        offset: u64,
        _size: u64,
    ) -> Result<Self> {
        let name = name.to_string();
        let node_offset = offset;
        let objects = Vec::new();

        reader.seek(SeekFrom::Start(offset))?;
        let metadata_size = u32_be(reader)?;
        let file_size = u32_be(reader)? as i64;
        let version = u32_be(reader)?;
        let data_offset = u32_be(reader)? as i64;
        if version < 9 {
            return Ok(Self {
                name,
                node_offset,
                metadata_size,
                file_size,
                version,
                data_offset,
                endianness: 0,
                objects,
            });
        }

        let endianness = read_endian(reader)?;
        if version < 22 {
            return Ok(Self {
                name,
                node_offset,
                metadata_size,
                file_size,
                version,
                data_offset,
                endianness,
                objects,
            });
        }

        let metadata_size = u32_be(reader)?;
        let file_size = i64_be(reader)?;
        let data_offset = i64_be(reader)?;
        reader.seek(SeekFrom::Current(8))?;

        let objects = Self::parse_objects(reader)?;

        Ok(Self {
            name,
            node_offset,
            metadata_size,
            file_size,
            version,
            data_offset,
            endianness,
            objects,
        })
    }

    fn parse_objects(reader: &mut (impl BufRead + Seek)) -> Result<Vec<ObjectData>> {
        let mut out = Vec::new();
        let base_pos = reader.stream_position()?;

        let unity_ver = cstring(reader)?;
        let unity_ver: Vec<u32> = unity_ver
            .split('.')
            .take(2)
            .flat_map(|v| v.parse::<u32>())
            .collect();
        let unity_version = (unity_ver[0], unity_ver[1]);
        let _target_platform = i32_le(reader)?;
        let enable_type_tree = byte(reader)? != 0;

        let type_count = i32_le(reader)?;
        let mut types = Vec::new();
        for _ in 0..type_count {
            let class_id = i32_le(reader)?;
            let _is_stripped = byte(reader)?;
            let _script_type_index = i16_le(reader)?;
            if class_id == 114 {
                reader.seek(SeekFrom::Current(16))?;
            }
            reader.seek(SeekFrom::Current(16))?;

            if enable_type_tree {
                let node_count = u32_le(reader)? as i64;
                let string_buffer_size = u32_le(reader)? as i64;
                reader.seek(SeekFrom::Current((node_count * 32) + string_buffer_size))?;
                let dep_count = i32_le(reader)? as i64;
                reader.seek(SeekFrom::Current(dep_count * 4))?;
            }

            types.push(class_id);
        }

        let object_count = i32_le(reader)?;
        for _ in 0..object_count {
            align4(reader, base_pos)?;
            let path_id = i64_le(reader)?;
            let byte_start = i64_le(reader)?;
            let byte_size = u32_le(reader)?;
            let type_id = i32_le(reader)?;
            let class_id = types
                .get(type_id as usize)
                .ok_or_else(|| {
                    anyhow!("type id {} out of bounds (total: {})", type_id, types.len())
                })?
                .to_owned();

            out.push(ObjectData {
                unity_version,
                path_id,
                byte_start,
                byte_size,
                class_id,
            })
        }

        Ok(out)
    }
}

fn read_endian(reader: &mut impl BufRead) -> Result<u8> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}
