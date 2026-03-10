use anyhow::Result;
use std::io::{BufRead, Cursor, Seek, SeekFrom};

use super::header::BundleHeader;

pub struct StorageBlock {
    pub decompressed_size: u32,
    pub compressed_size: u32,
    pub flags: u16,
}

pub struct DirectoryNode {
    pub offset: u64,
    pub size: u64,
    pub flags: u32,
    pub name: String,
}

pub struct BlockInfo {
    pub blocks: Vec<StorageBlock>,
    pub nodes: Vec<DirectoryNode>,
}

impl BlockInfo {
    pub fn parse(reader: &mut (impl BufRead + Seek), header: &BundleHeader) -> Result<BlockInfo> {
        if header.flags & 0x80 != 0 {
            let pos = header.file_size as u64 - header.compressed_blocks_size as u64;
            reader.seek(SeekFrom::Start(pos))?;
        } else {
            let pos = reader.stream_position()?;
            let aligned = (pos + 15) & !15;
            if aligned > pos {
                reader.seek(SeekFrom::Start(aligned))?;
            }
        }

        let mut compressed = vec![0u8; header.compressed_blocks_size as usize];
        reader.read_exact(&mut compressed)?;
        let decompressed =
            lz4_flex::decompress(&compressed, header.decompressed_blocks_size as usize)?;

        let mut cur = Cursor::new(decompressed);
        cur.seek(SeekFrom::Current(16))?;

        let block_count = super::read_u32(&mut cur)?;
        let mut blocks = Vec::new();
        for _ in 0..block_count {
            let decompressed_size = super::read_u32(&mut cur)?;
            let compressed_size = super::read_u32(&mut cur)?;
            let flags = super::read_u16(&mut cur)?;
            blocks.push(StorageBlock {
                decompressed_size,
                compressed_size,
                flags,
            });
        }

        let node_count = super::read_u32(&mut cur)?;
        let mut nodes = Vec::new();
        for _ in 0..node_count {
            let offset = super::read_u64(&mut cur)?;
            let size = super::read_u64(&mut cur)?;
            let flags = super::read_u32(&mut cur)?;
            let name = super::read_cstring(&mut cur)?;
            nodes.push(DirectoryNode {
                offset,
                size,
                flags,
                name,
            });
        }

        Ok(BlockInfo { blocks, nodes })
    }
}
