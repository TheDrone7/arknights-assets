mod block;
mod header;
mod serialized;

pub mod asset;
pub mod read;

use anyhow::{Context, Result};
use indicatif::HumanBytes;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, Write};
use std::path::Path;

use asset::{AudioClip, AudioData, ImageData, MonoBehaviour, Sprite, TextAsset, Texture2D};
use block::BlockInfo;
use header::BundleHeader;
use serialized::SerializedFile;

pub struct UnityBundle {
    pub header: BundleHeader,
    pub info: BlockInfo,
}

impl UnityBundle {
    pub fn parse(reader: &mut (impl BufRead + Seek)) -> Result<Self> {
        let header = BundleHeader::parse(reader)?;
        let info = BlockInfo::parse(reader, &header)?;

        Ok(Self { header, info })
    }

    pub fn decompress(
        &self,
        reader: &mut (impl BufRead + Seek),
        output: &mut impl Write,
    ) -> Result<usize> {
        self.info.decompress(reader, output, &self.header)
    }

    pub fn get_serialized(
        &self,
        reader: &mut (impl BufRead + Seek),
    ) -> Result<Vec<SerializedFile>> {
        let mut out = Vec::new();

        for node in &self.info.nodes {
            if !node.name.contains('.') {
                out.push(SerializedFile::parse(
                    &node.name,
                    reader,
                    node.offset,
                    node.size,
                )?);
            }
        }

        Ok(out)
    }

    pub fn process(path: &Path) -> Result<()> {
        let mut reader = BufReader::new(File::open(path)?);
        let bundle = Self::parse(&mut reader)
            .with_context(|| format!("SKIP [bad asset bundle] | [{}]", path.display(),))?;

        println!(
            "File: {} ({} blocks; {} nodes)",
            path.display(),
            bundle.info.blocks.len(),
            bundle.info.nodes.len()
        );

        let dec_path = path.with_extension("dec");
        let dec_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dec_path)?;
        let mut dec_writer = BufWriter::new(dec_file);
        let bytes_size = bundle
            .decompress(&mut reader, &mut dec_writer)
            .with_context(|| {
                format!(
                    "SKIP [bad compression] | [{}] -> {}",
                    path.display(),
                    dec_path.display()
                )
            })?;
        dec_writer.flush()?;
        println!(
            "  Decompressed: [{}] ({})",
            dec_path.display(),
            HumanBytes(bytes_size as u64)
        );

        let mut dec_reader = BufReader::new(File::open(&dec_path)?);
        for sf in bundle.get_serialized(&mut dec_reader)? {
            println!(
                "  Serialized: [{}] (v{}; {} objects)",
                sf.name,
                sf.version,
                sf.objects.len()
            );

            for obj in &sf.objects {
                let abs = sf.node_offset + (sf.data_offset + obj.byte_start) as u64;
                match obj.class_id {
                    28 => {
                        let texture =
                            Texture2D::parse(&mut dec_reader, abs).with_context(|| {
                                format!("Texture2D :: path_id = {}; abs = {}", obj.path_id, abs)
                            })?;
                        match texture.image {
                            ImageData::Inline(data) => {
                                println!(
                                    "    TEXTURE2D: [{}] ({}x{} | {})",
                                    texture.name,
                                    texture.width,
                                    texture.height,
                                    HumanBytes(data.len() as u64)
                                );
                            }
                            ImageData::Streaming { path, offset, size } => {
                                println!(
                                    "    TEXTURE2D: [{}] ({}x{} | {}@{}+{})",
                                    texture.name, texture.width, texture.height, path, offset, size
                                );
                            }
                        }
                    }
                    49 => {
                        let text = TextAsset::parse(&mut dec_reader, abs).with_context(|| {
                            format!("TextAsset :: path_id = {}; abs = {}", obj.path_id, abs)
                        })?;
                        println!(
                            "    TEXT: [{}] ({})",
                            text.name,
                            HumanBytes(text.data.len() as u64)
                        );
                    }
                    83 => {
                        let audio = AudioClip::parse(&mut dec_reader, abs).with_context(|| {
                            format!("AudioClip :: path_id = {}; abs = {}", obj.path_id, abs)
                        })?;
                        match audio.audio {
                            AudioData::Inline(data) => {
                                println!(
                                    "    AUDIO: [{}] ({}ch; {}Hz; {}s | {})",
                                    audio.name,
                                    audio.channels,
                                    audio.frequency,
                                    audio.length,
                                    HumanBytes(data.len() as u64)
                                );
                            }
                            AudioData::Streaming { path, offset, size } => {
                                println!(
                                    "    AUDIO: [{}] ({}ch; {}Hz; {}s | {}@{}+{}",
                                    audio.name,
                                    audio.channels,
                                    audio.frequency,
                                    audio.length,
                                    path,
                                    offset,
                                    size
                                );
                            }
                        }
                    }
                    114 => {
                        let mono = MonoBehaviour::parse(&mut dec_reader, abs, obj.byte_size)
                            .with_context(|| {
                                format!(
                                    "MonoBehaviour :: path_id = {}; abs = {}; byte_size = {}",
                                    obj.path_id, abs, obj.byte_size
                                )
                            })?;
                        println!(
                            "    MONO BEHAVIOUR: [{}] ({})",
                            mono.name,
                            HumanBytes(mono.data.len() as u64)
                        );
                    }
                    213 => {
                        let sprite = Sprite::parse(&mut dec_reader, abs).with_context(|| {
                            format!("Sprite :: path_id = {}; abs = {}", obj.path_id, abs)
                        })?;
                        println!(
                            "    SPRITE: [{}] ({:?} + {:?} => {:?})",
                            sprite.name, sprite.texture, sprite.alpha_texture, sprite.texture_rect
                        );
                    }
                    _ => {}
                };
            }
        }

        println!("\n");
        Ok(())
    }
}
