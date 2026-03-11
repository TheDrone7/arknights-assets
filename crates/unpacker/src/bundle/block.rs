use anyhow::{Result, bail};
use std::io::{BufRead, Cursor, Seek, SeekFrom, Write};

use super::header::BundleHeader;
use super::read::*;
use crate::lz4inv;

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

        let block_count = u32_be(&mut cur)?;
        let mut blocks = Vec::new();
        for _ in 0..block_count {
            let decompressed_size = u32_be(&mut cur)?;
            let compressed_size = u32_be(&mut cur)?;
            let flags = u16_be(&mut cur)?;
            blocks.push(StorageBlock {
                decompressed_size,
                compressed_size,
                flags,
            });
        }

        let node_count = u32_be(&mut cur)?;
        let mut nodes = Vec::new();
        for _ in 0..node_count {
            let offset = u64_be(&mut cur)?;
            let size = u64_be(&mut cur)?;
            let flags = u32_be(&mut cur)?;
            let name = cstring(&mut cur)?;
            nodes.push(DirectoryNode {
                offset,
                size,
                flags,
                name,
            });
        }

        Ok(BlockInfo { blocks, nodes })
    }

    pub fn decompress(
        &self,
        reader: &mut (impl BufRead + Seek),
        output: &mut impl Write,
        header: &BundleHeader,
    ) -> Result<usize> {
        if header.flags & 0x200 != 0 {
            let pos = reader.stream_position()?;
            let aligned = (pos + 15) & !15;
            if aligned > pos {
                reader.seek(SeekFrom::Start(aligned))?;
            }
        }
        let mut total_decompressed = 0;

        for storage_block in &self.blocks {
            let mut buf = vec![0u8; storage_block.compressed_size as usize];
            reader.read_exact(&mut buf)?;

            match storage_block.flags & 0x3F {
                0 => {
                    output.write_all(&buf)?;
                    total_decompressed += buf.len();
                }
                2 | 3 => {
                    let dec = lz4_flex::decompress(&buf, storage_block.decompressed_size as usize)?;
                    output.write_all(&dec)?;
                    total_decompressed += dec.len();
                }
                4 => {
                    let dec = lz4inv::decompress(&buf, storage_block.decompressed_size as usize)?;
                    output.write_all(&dec)?;
                    total_decompressed += dec.len();
                }
                t => bail!("unsupported block compression type: {}", t),
            };
        }

        Ok(total_decompressed)
    }
}
