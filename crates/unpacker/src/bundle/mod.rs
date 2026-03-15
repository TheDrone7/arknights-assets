mod block;
mod header;
mod serialized;
mod typetree;

pub mod asset;
pub mod read;

use anyhow::{Context, Result};
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

    pub fn process(path: &Path, out_base: &Path) -> Result<()> {
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
        let _decompressed_size = bundle
            .decompress(&mut reader, &mut dec_writer)
            .with_context(|| {
                format!(
                    "SKIP [bad compression] | [{}] -> {}",
                    path.display(),
                    dec_path.display()
                )
            })?;
        dec_writer.flush()?;
        let mut dec_reader = BufReader::new(File::open(&dec_path)?);
        let serialized_files = bundle.get_serialized(&mut dec_reader)?;
        let total_objects: usize = serialized_files.iter().map(|f| f.objects.len()).sum();
        let bundle_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("bundle");
        let out_dir = if total_objects <= 1 {
            out_base.to_path_buf()
        } else {
            out_base.join(bundle_stem)
        };

        let mut i = 0;

        for sf in &serialized_files {
            for obj in &sf.objects {
                let abs = sf.node_offset + (sf.data_offset + obj.byte_start) as u64;
                match obj.class_id {
                    28 => {
                        let texture = Texture2D::parse(obj.unity_version, &mut dec_reader, abs)
                            .with_context(|| {
                                format!("Texture2D :: path_id = {}; abs = {}", obj.path_id, abs)
                            })?;
                        let ress_base = if let ImageData::Streaming { path: p, .. } = &texture.image
                        {
                            let name = Path::new(p)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(p.as_str());
                            bundle
                                .info
                                .nodes
                                .iter()
                                .find(|n| n.name == name)
                                .map(|n| n.offset)
                        } else {
                            None
                        };
                        texture
                            .extract(&out_dir, &dec_path, ress_base)
                            .with_context(|| format!("Texture2D::extract '{}'", &texture.name))?;
                    }
                    49 => {
                        let text = TextAsset::parse(&mut dec_reader, abs).with_context(|| {
                            format!("TextAsset :: path_id = {}; abs = {}", obj.path_id, abs)
                        })?;
                        text.extract(&out_dir)
                            .with_context(|| format!("TextAsset::extract '{}'", &text.name))?;
                    }
                    83 => {
                        let audio = AudioClip::parse(&mut dec_reader, abs).with_context(|| {
                            format!("AudioClip :: path_id = {}; abs = {}", obj.path_id, abs)
                        })?;
                        let ress_base = if let AudioData::Streaming { path: p, .. } = &audio.audio {
                            let name = Path::new(&p)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(p.as_str());
                            bundle
                                .info
                                .nodes
                                .iter()
                                .find(|n| n.name == name)
                                .map(|n| n.offset)
                        } else {
                            None
                        };
                        audio
                            .extract(&out_dir, &dec_path, ress_base)
                            .with_context(|| format!("AudioClip::extract '{}'", audio.name))?;
                    }
                    114 => {
                        let nodes = sf.type_nodes(obj.type_id);
                        let mono = MonoBehaviour::parse(nodes, &mut dec_reader, abs, obj.byte_size)
                            .with_context(|| {
                                format!(
                                    "MonoBehaviour :: path_id = {}; abs = {}; byte_size = {}",
                                    obj.path_id, abs, obj.byte_size
                                )
                            })?;
                        mono.extract(&out_dir, i).with_context(|| {
                            format!("MonoBehaviour::extract '{}' ({})", mono.name, i)
                        })?;
                    }
                    213 => {
                        let _sprite = Sprite::parse(&mut dec_reader, abs).with_context(|| {
                            format!("Sprite :: path_id = {}; abs = {}", obj.path_id, abs)
                        })?;
                        // println!(
                        //     "    SPRITE: [{}] ({:?} + {:?} => {:?})",
                        //     sprite.name, sprite.texture, sprite.alpha_texture, sprite.texture_rect
                        // );
                    }
                    _ => {}
                };
                i += 1;
            }
        }

        Ok(())
    }
}
