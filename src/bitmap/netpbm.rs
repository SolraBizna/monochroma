use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use anyhow::{anyhow, Context};

use super::*;

fn peek(reader: &mut impl BufRead) -> anyhow::Result<u8> {
    let buf = reader.fill_buf()?;
    if buf.is_empty() {
        Err(anyhow!("Unexpected end of file."))
    } else {
        Ok(buf[0])
    }
}

fn munch_whitespace(reader: &mut impl BufRead) -> anyhow::Result<()> {
    munch_one_whitespace(reader)?;
    loop {
        if munch_comment(reader)? {
            // om nom nom
        } else if peek(reader)?.is_ascii_whitespace() {
            reader.consume(1);
        } else {
            break;
        }
    }
    Ok(())
}

fn munch_one_whitespace(reader: &mut impl BufRead) -> anyhow::Result<()> {
    while munch_comment(reader)? {}
    if peek(reader)?.is_ascii_whitespace() {
        reader.consume(1);
        Ok(())
    } else {
        Err(anyhow!("Expected whitespace, found something else."))
    }
}

fn munch_comment(reader: &mut impl BufRead) -> anyhow::Result<bool> {
    if peek(reader)? == b'#' {
        reader.consume(1);
        while peek(reader)? != b'\n' {
            reader.consume(1);
        }
        if peek(reader)? == b'\n' {
            reader.consume(1);
            Ok(true)
        } else {
            unreachable!()
        }
    } else {
        Ok(false)
    }
}

fn munch_number(reader: &mut impl BufRead) -> anyhow::Result<u32> {
    let mut n = 0u32;
    if !peek(reader)?.is_ascii_digit() {
        return Err(anyhow!("Number did not begin with an ASCII digit."));
    }
    while let Ok(digit) = peek(reader) {
        if !digit.is_ascii_digit() {
            break;
        }
        n = n
            .checked_mul(10)
            .and_then(|n| n.checked_add((digit - b'0') as u32))
            .ok_or_else(|| anyhow!("Arithmetic overflow."))?;
        reader.consume(1);
    }
    Ok(n)
}

impl Bitmap {
    /// Save this Bitmap to a binary (P4) Portable BitMap.
    ///
    /// See: <https://netpbm.sourceforge.net/doc/pbm.html>
    pub fn write_binary_pbm(
        &self,
        mut writer: impl Write,
        comments: &str,
    ) -> std::io::Result<()> {
        writeln!(writer, "P4")?;
        if !comments.is_empty() {
            for line in comments.lines() {
                writeln!(writer, "#{line}")?;
            }
        }
        writeln!(writer, "{} {}", self.width, self.height)?;
        let bytes = self.to_bytes();
        writer.write_all(&bytes)
    }
    /// Save this Bitmap to an ASCII (P1) Portable BitMap.
    ///
    /// See: <https://netpbm.sourceforge.net/doc/pbm.html>
    pub fn write_ascii_pbm(
        &self,
        writer: impl Write,
        comments: &str,
    ) -> std::io::Result<()> {
        let mut writer = BufWriter::new(writer);
        writeln!(writer, "P1")?;
        if !comments.is_empty() {
            for line in comments.lines() {
                writeln!(writer, "#{line}")?;
            }
        }
        writeln!(writer, "{} {}", self.width, self.height)?;
        let mut i = 0;
        for _ in 0..self.height {
            for x in 0..self.width {
                write!(
                    writer,
                    "{}",
                    (self.bits[i] >> (31 ^ (x as usize % BITMAP_WORD_BITS)))
                        & 1
                )?;
                if x != self.width - 1 {
                    write!(writer, " ")?;
                }
                if x as usize % BITMAP_WORD_BITS == (BITMAP_WORD_BITS) - 1 {
                    i += 1;
                }
            }
            if self.width as usize % BITMAP_WORD_BITS != 0 {
                i += 1;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
    /// Load a new Bitmap from any netpbm format (pbm, pgm, ppm, or pam).
    ///
    /// See: <https://netpbm.sourceforge.net/doc/index.html>
    pub fn read_netpbm(reader: impl Read) -> anyhow::Result<Bitmap> {
        let mut reader = BufReader::new(reader);
        let mut kind = [0; 2];
        reader
            .read_exact(&mut kind)
            .context("Couldn't read header.")?;
        if kind[0] != b'P' {
            return Err(anyhow!("Input is not a netpbm image."));
        }
        match kind[1] {
            b'1' => todo!("P1"),
            b'2' => todo!("P2"),
            b'3' => todo!("P3"),
            b'4' => read_p4(&mut reader),
            b'5' => todo!("P5"),
            b'6' => todo!("P6"),
            b'7' => todo!("P7"),
            _ => Err(anyhow!("Input is not a netpbm image.")),
        }
    }
}

fn read_p4(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_one_whitespace(reader)?;
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
    Ok(Bitmap {
        width,
        height,
        pitch_words,
        bits,
    })
}
