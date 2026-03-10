use anyhow::{Result, anyhow, ensure};

pub fn decompress(compressed: &[u8], decompressed_size: usize) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(decompressed_size);
    let mut i = 0;

    while out.len() < decompressed_size {
        ensure!(i < compressed.len(), "LZ4Inv: Input truncated");
        let token = compressed[i];
        i += 1;

        let mut lit_len = (token & 0x0F) as usize;
        if lit_len == 15 {
            loop {
                ensure!(i < compressed.len(), "LZ4Inv: Lit len truncated");
                let e = compressed[i] as usize;
                i += 1;
                lit_len += e;
                if e < 255 {
                    break;
                }
            }
        }

        ensure!(
            i + lit_len <= compressed.len(),
            "LZ4Inv: literal out of bounds"
        );
        out.extend_from_slice(&compressed[i..i + lit_len]);
        i += lit_len;

        if out.len() >= decompressed_size {
            break;
        }

        ensure!(i + 2 <= compressed.len(), "LZ4Inv: offset truncated");
        let offset = ((compressed[i] as usize) << 8) | (compressed[i + 1] as usize);
        i += 2;
        ensure!(offset != 0, "LZ4Inv: zero offset");

        let mut match_len = (token >> 4) as usize + 4;
        if (token >> 4) == 15 {
            loop {
                ensure!(i < compressed.len(), "LZ4Inv: match len truncated");
                let e = compressed[i] as usize;
                i += 1;
                match_len += e;

                if e < 255 {
                    break;
                }
            }
        }

        let match_pos = out.len().checked_sub(offset).ok_or_else(|| {
            anyhow!("the offset to copy is not contained in the decompressed buffer")
        })?;
        for k in 0..match_len {
            let b = out[match_pos + k];
            out.push(b);
        }
    }

    Ok(out)
}
