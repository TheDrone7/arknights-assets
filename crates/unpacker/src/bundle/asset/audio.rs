use anyhow::{Context, Result};
use std::fs::{File, create_dir_all, write};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::bundle::read::*;

pub enum AudioData {
    Inline(Vec<u8>),
    Streaming {
        path: String,
        offset: u64,
        size: u64,
    },
}

pub struct AudioClip {
    pub name: String,
    pub channels: i32,
    pub frequency: i32,
    pub length: f32,
    pub compression_format: i32,
    pub audio: AudioData,
}

impl AudioClip {
    pub fn parse(reader: &mut (impl BufRead + Seek), abs_offset: u64) -> Result<Self> {
        reader.seek(SeekFrom::Start(abs_offset))?;
        let name_bytes = aligned_bytes(reader)?;
        let name = String::from_utf8(name_bytes)?;

        let _load_type = i32_le(reader)?;
        let channels = i32_le(reader)?;
        let frequency = i32_le(reader)?;
        let _bits_per_sample = i32_le(reader)?;
        let length = f32_le(reader)?;
        let _is_tracker_format = byte(reader)?;
        align4(reader, 0)?;
        let _sound_index = i32_le(reader)?;
        let _preload_audio_data = byte(reader)?;
        let _load_in_background = byte(reader)?;
        let _legacy_3d = byte(reader)?;
        align4(reader, 0)?;

        let source_bytes = aligned_bytes(reader)?;
        let source = String::from_utf8(source_bytes)?;
        let offset = i64_le(reader)? as u64;
        let size = i64_le(reader)? as u64;
        let compression_format = i32_le(reader)?;

        let audio = if source.is_empty() {
            let mut buf = vec![0u8; size as usize];
            reader.read_exact(&mut buf)?;
            AudioData::Inline(buf)
        } else {
            AudioData::Streaming {
                path: source,
                offset,
                size,
            }
        };

        Ok(Self {
            name,
            channels,
            frequency,
            length,
            compression_format,
            audio,
        })
    }

    pub fn extract(&self, dir: &Path, dec_path: &Path, ress_base: Option<u64>) -> Result<PathBuf> {
        create_dir_all(dir)?;
        let out_path = dir.join(&self.name);

        match &self.audio {
            AudioData::Inline(data) => {
                write(&out_path, data)?;
            }
            AudioData::Streaming { offset, size, .. } => {
                let base =
                    ress_base.with_context(|| format!("No .resS node for '{}'", &self.name))?;
                let mut f = BufReader::new(File::open(dec_path)?);
                f.seek(SeekFrom::Start(base + offset))?;
                let mut data = vec![0u8; *size as usize];
                f.read_exact(&mut data)?;
                write(&out_path, &data)?;
            }
        }

        Ok(out_path)
    }
}
