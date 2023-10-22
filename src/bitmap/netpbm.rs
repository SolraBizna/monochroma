use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use anyhow::{anyhow, Context};

use super::*;

fn peek(reader: &mut impl BufRead) -> anyhow::Result<u8> {
    let buf = reader.fill_buf()?;
    if buf.is_empty() {
        Err(anyhow!("Unexpected end of file"))
    } else {
        Ok(buf[0])
    }
}

fn munch_optional_whitespace(reader: &mut impl BufRead) -> anyhow::Result<()> {
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

fn munch_mandatory_whitespace(
    reader: &mut impl BufRead,
) -> anyhow::Result<()> {
    munch_one_whitespace(reader)?;
    munch_optional_whitespace(reader)
}

fn munch_one_whitespace(reader: &mut impl BufRead) -> anyhow::Result<()> {
    while munch_comment(reader)? {}
    if peek(reader)?.is_ascii_whitespace() {
        reader.consume(1);
        Ok(())
    } else {
        Err(anyhow!("Expected whitespace, found something else"))
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
        return Err(anyhow!("Number did not begin with an ASCII digit"));
    }
    while let Ok(digit) = peek(reader) {
        if !digit.is_ascii_digit() {
            break;
        }
        n = n
            .checked_mul(10)
            .and_then(|n| n.checked_add((digit - b'0') as u32))
            .ok_or_else(|| anyhow!("Arithmetic overflow"))?;
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
                    (self.words[i] >> (31 ^ (x as usize % BITMAP_WORD_BITS)))
                        & 1
                )?;
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
    /// Load a new Bitmap from any netpbm format (pbm, pgm, ppm, NOT pam). For
    /// non-bitmap input, the pixel is marked (1) if the *sum of all channels*
    /// is strictly less than half of the maximum value it could possibly be.
    /// If you have a mark-is-bright system, you will need to invert this
    /// bitmap. This can be done efficiently by filling its bounds rectangle
    /// with `ModeXor(())`.
    ///
    /// See: <https://netpbm.sourceforge.net/doc/index.html>
    pub fn read_netpbm(reader: impl Read) -> anyhow::Result<Bitmap> {
        let mut reader = BufReader::new(reader);
        let mut kind = [0; 2];
        reader
            .read_exact(&mut kind)
            .context("Couldn't read header")?;
        if kind[0] != b'P' {
            return Err(anyhow!("Input is not a netpbm image"));
        }
        match kind[1] {
            b'1' => read_p1(&mut reader),
            b'2' => read_p2(&mut reader),
            b'3' => read_p3(&mut reader),
            b'4' => read_p4(&mut reader),
            b'5' => read_p5(&mut reader),
            b'6' => read_p6(&mut reader),
            b'7' => Err(anyhow!(
                "Image is a PAM or an xv thumbnail; we don't support either"
            )),
            _ => Err(anyhow!("Input is not a netpbm image")),
        }
    }
}

macro_rules! read_pixels {
    ($width:ident, $height:ident, $code:block) => {{
        let words_per_row = get_word_pitch($width);
        let mut words = Vec::with_capacity((words_per_row * $height) as usize);
        for _ in 0..$height {
            for _ in 0..($width - $width % BITMAP_WORD_BITS as u32)
                / BITMAP_WORD_BITS as u32
            {
                let mut word = 0;
                let mut bit = !0 ^ ((!0) >> 1);
                for _ in 0..BITMAP_WORD_BITS {
                    if $code {
                        word |= bit
                    }
                    bit >>= 1;
                }
                words.push(word);
            }
            let mut word = 0;
            let mut bit = !0 ^ ((!0) >> 1);
            for _ in $width - $width % BITMAP_WORD_BITS as u32..$width {
                if $code {
                    word |= bit
                }
                bit >>= 1;
            }
            words.push(word);
        }
        (words_per_row, words)
    }};
}

/// ASCII PBM
fn read_p1(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_mandatory_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_one_whitespace(reader)?;
    let (words_per_row, words) = read_pixels!(width, height, {
        munch_optional_whitespace(reader)?;
        match peek(reader)? {
            b'0' => {
                reader.consume(1);
                false
            }
            b'1' => {
                reader.consume(1);
                true
            }
            _ => {
                return Err(anyhow!(
                "Unexpected non-comment, non-whitespace, non-zero-or-one byte"
            ))
            }
        }
    });
    Ok(Bitmap {
        width,
        height,
        words_per_row,
        words,
    })
}

/// ASCII PGM
fn read_p2(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_mandatory_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let maxval = munch_number(reader)?;
    if maxval > 65535 {
        return Err(anyhow!("maxval greater than 65535 specified"));
    } else if maxval == 0 {
        return Err(anyhow!("zero maxval specified"));
    }
    let halfie = (maxval + 1) / 2;
    let (words_per_row, words) = read_pixels!(width, height, {
        munch_mandatory_whitespace(reader)?;
        let num = munch_number(reader)?;
        if num > maxval {
            return Err(anyhow!("pixel value exceeding maxval specified"));
        } else {
            num < halfie
        }
    });
    Ok(Bitmap {
        width,
        height,
        words_per_row,
        words,
    })
}

/// ASCII PPM
fn read_p3(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_mandatory_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let maxval = munch_number(reader)?;
    if maxval > 65535 {
        return Err(anyhow!("maxval greater than 65535 specified"));
    } else if maxval == 0 {
        return Err(anyhow!("zero maxval specified"));
    }
    let maxval = maxval * 3;
    let halfie = (maxval + 1) / 2;
    let (words_per_row, words) = read_pixels!(width, height, {
        munch_mandatory_whitespace(reader)?;
        let num = munch_number(reader)?
            + munch_mandatory_whitespace(reader)
                .and_then(|_| munch_number(reader))?
            + munch_mandatory_whitespace(reader)
                .and_then(|_| munch_number(reader))?;
        if num > maxval {
            return Err(anyhow!("pixel value exceeding maxval specified"));
        } else {
            num < halfie
        }
    });
    Ok(Bitmap {
        width,
        height,
        words_per_row,
        words,
    })
}

/// Binary PBM
fn read_p4(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_mandatory_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_one_whitespace(reader)?;
    let words_per_row = get_word_pitch(width);
    let mut words = Vec::with_capacity((words_per_row * height) as usize);
    let mut buf = vec![0; (width as usize + 7) / 8];
    for _ in 0..height {
        reader
            .read_exact(&mut buf[..])
            .context("Couldn't read bitmap bits")?;
        for chunk in buf.chunks(BITMAP_WORD_BYTES) {
            words.push(BitmapWord::from_be_bytes([
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
        words_per_row,
        words,
    })
}

/// Binary PGM
fn read_p5(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_mandatory_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let maxval = munch_number(reader)?;
    munch_one_whitespace(reader)?;
    if maxval > 65535 {
        return Err(anyhow!("maxval greater than 65535 specified"));
    } else if maxval == 0 {
        return Err(anyhow!("zero maxval specified"));
    }
    let halfie = (maxval + 1) / 2;
    let (words_per_row, words) = if maxval > 255 {
        read_pixels!(width, height, {
            let mut buf = [0; 2];
            reader.read_exact(&mut buf)?;
            let num = u16::from_be_bytes(buf) as u32;
            if num > maxval {
                return Err(anyhow!("pixel value exceeding maxval specified"));
            } else {
                num < halfie
            }
        })
    } else {
        read_pixels!(width, height, {
            let mut buf = [0; 1];
            reader.read_exact(&mut buf)?;
            let num = buf[0] as u32;
            if num > maxval {
                return Err(anyhow!("pixel value exceeding maxval specified"));
            } else {
                num < halfie
            }
        })
    };
    Ok(Bitmap {
        width,
        height,
        words_per_row,
        words,
    })
}

/// Binary PPM
fn read_p6(reader: &mut impl BufRead) -> anyhow::Result<Bitmap> {
    munch_mandatory_whitespace(reader)?;
    let width = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let height = munch_number(reader)?;
    munch_mandatory_whitespace(reader)?;
    let maxval = munch_number(reader)?;
    munch_one_whitespace(reader)?;
    if maxval > 65535 {
        return Err(anyhow!("maxval greater than 65535 specified"));
    } else if maxval == 0 {
        return Err(anyhow!("zero maxval specified"));
    }
    let maxval = maxval * 3;
    let halfie = (maxval + 1) / 2;
    let (words_per_row, words) = if maxval > 255 * 3 {
        read_pixels!(width, height, {
            let mut buf = [0; 6];
            reader.read_exact(&mut buf)?;
            let num = u16::from_be_bytes([buf[0], buf[1]]) as u32
                + u16::from_be_bytes([buf[2], buf[3]]) as u32
                + u16::from_be_bytes([buf[4], buf[5]]) as u32;
            if num > maxval {
                return Err(anyhow!("pixel value exceeding maxval specified"));
            } else {
                num < halfie
            }
        })
    } else {
        read_pixels!(width, height, {
            let mut buf = [0; 3];
            reader.read_exact(&mut buf)?;
            let num = buf[0] as u32 + buf[1] as u32 + buf[2] as u32;
            if num > maxval {
                return Err(anyhow!("pixel value exceeding maxval specified"));
            } else {
                num < halfie
            }
        })
    };
    Ok(Bitmap {
        width,
        height,
        words_per_row,
        words,
    })
}
