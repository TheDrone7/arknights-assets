mod block;
mod header;
mod serialized;

pub mod asset;
pub mod read;

use anyhow::Result;
use std::io::{BufRead, Seek, Write};

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
}
