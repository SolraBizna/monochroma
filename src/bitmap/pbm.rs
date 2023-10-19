use std::io::{BufRead, BufReader, Read};

use anyhow::{anyhow, Context};

use super::*;

fn munch_whitespace(buf: &[u8]) -> Option<&[u8]> {
    let mut buf = munch_one_whitespace(buf)?;
    loop {
        if let Some(munched) = munch_comment(buf) {
            buf = munched;
        } else if buf.first().map(u8::is_ascii_whitespace).unwrap_or(false) {
            buf = &buf[1..];
        } else {
            break;
        }
    }
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

fn munch_one_whitespace(mut buf: &[u8]) -> Option<&[u8]> {
    while let Some(munched) = munch_comment(buf) {
        buf = munched;
    }
    if buf.first().map(u8::is_ascii_whitespace).unwrap_or(false) {
        buf = &buf[1..];
    }
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

fn munch_comment(mut buf: &[u8]) -> Option<&[u8]> {
    if buf.starts_with(b"#") {
        buf = &buf[1..];
        while !buf.is_empty() && !buf.starts_with(b"\n") {
            buf = &buf[1..];
        }
        if buf.starts_with(b"\n") {
            buf = &buf[1..];
        }
        if !buf.is_empty() {
            return Some(buf);
        }
    }
    None
}

fn munch_number(mut buf: &[u8]) -> Option<(u32, &[u8])> {
    let mut n = 0u32;
    assert!(!buf.is_empty());
    assert!(buf[0].is_ascii_digit());
    while !buf.is_empty() && buf[0].is_ascii_digit() {
        n = n.checked_mul(10)?;
        n = n.checked_add((buf[0] - b'0') as u32)?;
        buf = &buf[1..];
    }
    if buf.is_empty() {
        None
    } else {
        Some((n, buf))
    }
}

impl Bitmap {
    /// Create a new Bitmap from a binary Portable BitMap.
    ///
    /// See: <https://netpbm.sourceforge.net/doc/pbm.html>
    pub fn from_pbm(i: impl Read) -> anyhow::Result<Bitmap> {
        let mut reader = BufReader::new(i);
        let origbuf = reader.fill_buf().context("Couldn't read header.")?;
        if origbuf.len() < 7 {
            return Err(anyhow!("File too short to be a PBM"));
        }
        if !origbuf.starts_with(b"P4") {
            return Err(anyhow!("File is not a PBM"));
        }
        let redbuf = &origbuf[2..];
        let redbuf = munch_whitespace(redbuf)
            .ok_or_else(|| anyhow!("Corrupted header in PBM"))?;
        let (width, redbuf) = munch_number(redbuf)
            .ok_or_else(|| anyhow!("Corrupted header in PBM"))?;
        let redbuf = munch_whitespace(redbuf)
            .ok_or_else(|| anyhow!("Corrupted header in PBM"))?;
        let (height, redbuf) = munch_number(redbuf)
            .ok_or_else(|| anyhow!("Corrupted header in PBM"))?;
        let redbuf = munch_one_whitespace(redbuf)
            .ok_or_else(|| anyhow!("Corrupted header in PBM"))?;
        let len = origbuf.len() - redbuf.len();
        reader.consume(len);
        let pitch_words = get_word_pitch(width);
        let mut bits = Vec::with_capacity((pitch_words * height) as usize);
        let mut buf = vec![0; (width as usize + 7) / 8];
        for _ in 0..height {
            reader
                .read_exact(&mut buf[..])
                .context("Couldn't read bitmap bits")?;
            for chunk in buf.chunks(BITMAP_WORD_BYTES) {
                bits.push(BitmapWord::from_be_bytes([
                    chunk.first().copied().unwrap_or(0),
                    chunk.get(1).copied().unwrap_or(0),
                    chunk.get(2).copied().unwrap_or(0),
                    chunk.get(3).copied().unwrap_or(0),
                ]))
            }
        }
        reader.consume(len);
        Ok(Bitmap {
            width,
            height,
            pitch_words,
            bits,
        })
    }
}
