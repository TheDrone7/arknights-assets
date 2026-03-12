use anyhow::{Context, Result};
use std::fs::{File, create_dir_all, write};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::bundle::read::*;

pub enum ImageData {
    Inline(Vec<u8>),
    Streaming {
        path: String,
        offset: u64,
        size: u32,
    },
}

pub struct Texture2D {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub format: i32,
    pub image: ImageData,
}

impl Texture2D {
    pub fn parse(
        (major, minor): (u32, u32),
        reader: &mut (impl BufRead + Seek),
        abs_offset: u64,
    ) -> Result<Self> {
        reader.seek(SeekFrom::Start(abs_offset))?;
        let name_bytes = aligned_bytes(reader)?;
        let name = String::from_utf8(name_bytes)?;

        // Texture
        let _forced_fallback_format = i32_le(reader)?;
        let _downscale_fallback = byte(reader)?;
        let _is_alpha_channel_optional = byte(reader)?;
        align4(reader, 0)?;

        // Texture2D
        let width = i32_le(reader)?;
        let height = i32_le(reader)?;
        let _complete_image_size = i32_le(reader)?;
        let _mips_stripped = i32_le(reader)?;
        let texture_format = i32_le(reader)?;
        let _mip_count = i32_le(reader)?;
        let _is_readable = byte(reader)?;
        let _is_pre_processed = byte(reader)?;
        let _ignore_master_texture_limit = byte(reader)?;
        if major > 2022 || (major == 2022 && minor >= 2) {
            let _mipmap_limit_group_name = aligned_bytes(reader)?;
        }
        let _streaming_mipmaps = byte(reader)?;
        align4(reader, 0)?;
        let _streaming_mipmaps_priority = i32_le(reader)?;
        let _image_count = i32_le(reader)?;
        let _texture_dims = i32_le(reader)?;

        // GL Texture
        let _filter_mode = i32_le(reader)?;
        let _aniso = i32_le(reader)?;
        let _mip_bias = f32_le(reader)?;
        let _wrap_u = i32_le(reader)?;
        let _wrap_v = i32_le(reader)?;
        let _wrap_w = i32_le(reader)?;

        // Texture2D
        let _lightmap_format = i32_le(reader)?;
        let _color_space = i32_le(reader)?;
        let _platform_bytes = aligned_bytes(reader)?;

        let image_data_size = i32_le(reader)?;
        let image = if image_data_size == 0 {
            let stream_offset = i64_le(reader)?;
            let stream_size = u32_le(reader)?;
            let stream_path_bytes = aligned_bytes(reader)?;
            let stream_path = String::from_utf8(stream_path_bytes)?;
            ImageData::Streaming {
                path: stream_path,
                size: stream_size,
                offset: stream_offset as u64,
            }
        } else {
            let mut image_data = vec![0u8; image_data_size as usize];
            reader.read_exact(&mut image_data)?;
            ImageData::Inline(image_data)
        };

        Ok(Self {
            name,
            width,
            height,
            format: texture_format,
            image,
        })
    }

    pub fn extract(&self, dir: &Path, dec_path: &Path, ress_base: Option<u64>) -> Result<PathBuf> {
        create_dir_all(dir)?;
        let out_path = dir.join(&self.name);

        match &self.image {
            ImageData::Inline(data) => {
                write(&out_path, data)?;
            }
            ImageData::Streaming { offset, size, .. } => {
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
