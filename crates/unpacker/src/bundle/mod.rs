mod block;
mod header;
mod read;

use anyhow::Result;
use std::io::{BufRead, Seek, Write};

use block::BlockInfo;
use header::BundleHeader;

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
}
