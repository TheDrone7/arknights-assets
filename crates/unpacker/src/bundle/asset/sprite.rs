use anyhow::Result;
use std::io::{BufRead, Seek, SeekFrom};

use super::common::{PPtr, Rectangle};
use crate::bundle::read::*;

pub struct Sprite {
    pub name: String,
    pub texture: PPtr,
    pub alpha_texture: PPtr,
    pub texture_rect: Rectangle,
}

impl Sprite {
    pub fn parse(reader: &mut (impl BufRead + Seek), abs_offset: u64) -> Result<Self> {
        reader.seek(SeekFrom::Start(abs_offset))?;
        let name_bytes = aligned_bytes(reader)?;
        let name = String::from_utf8(name_bytes)?;

        // Sprite
        let _rect = Rectangle::parse(reader)?;
        let _offset = (f32_le(reader)?, f32_le(reader)?);
        let _border = Rectangle::parse(reader)?;
        let _pixel_to_units = f32_le(reader)?;
        let _pivot = (f32_le(reader)?, f32_le(reader)?);
        let _extrude = u32_le(reader)?;
        let _is_polygon = byte(reader)? != 0;
        align4(reader, 0)?;
        let mut _guid_buf = [0u8; 16];
        reader.read_exact(&mut _guid_buf)?;
        let _render_data_key = i64_le(reader)?;
        let atlas_tag_count = i32_le(reader)?;
        for _ in 0..atlas_tag_count {
            let _atlas_tag_bytes = aligned_bytes(reader)?;
        }
        let _sprite_atlas_file = PPtr::parse(reader)?;

        // Sprite render data
        let texture = PPtr::parse(reader)?;
        let alpha_texture = PPtr::parse(reader)?;

        let secondary_count = i32_le(reader)?;
        for _ in 0..secondary_count {
            let _secondary_pptr = PPtr::parse(reader)?;
            let _secondary_bytes = aligned_bytes(reader)?;
        }

        let submesh_count = i32_le(reader)?;
        for _ in 0..submesh_count {
            reader.seek(SeekFrom::Current(48))?;
        }

        let _index_buffer = aligned_bytes(reader)?;
        let _vertex_count = u32_le(reader)?;
        let channel_count = i32_le(reader)?;
        for _ in 0..channel_count {
            let _ch_stream = byte(reader)?;
            let _ch_offset = byte(reader)?;
            let _ch_format = byte(reader)?;
            let _ch_dimensions = byte(reader)?;
        }
        let _v_data = aligned_bytes(reader)?;
        let matrix_count = i32_le(reader)?;
        for _ in 0..matrix_count {
            let mut _matrix = [[0f32; 4]; 4];
            for i in 0..16 {
                let a = i % 4;
                let b = i / 4;
                _matrix[a][b] = f32_le(reader)?;
            }
        }

        let texture_rect = Rectangle::parse(reader)?;

        Ok(Self {
            name,
            texture,
            alpha_texture,
            texture_rect,
        })
    }
}
