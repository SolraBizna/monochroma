use super::*;

mod draw;

pub use draw::*;

#[cfg(feature = "font")]
mod font;
#[cfg(feature = "font")]
pub use font::*;

#[cfg(feature = "netpbm")]
mod netpbm;

/// A "word" within a 1-bit image. In the current version, this is a `u32`
/// containing 32 pixels. The most significant bit is the leftmost pixel, the
/// least significant bit is the rightmost pixel.
///
/// This type will not change without a major version bump.
pub type BitmapWord = u32;
/// Convenience constant: the number of bytes in a `BitmapWord`.
pub const BITMAP_WORD_BYTES: usize = std::mem::size_of::<BitmapWord>();
/// Convenience constant: the number of *bits* (and therefore pixels) in a
/// `BitmapWord`.
pub const BITMAP_WORD_BITS: usize = BITMAP_WORD_BYTES * 8;

/// Convenience function: Calculate the number of words required to store one
/// N-pixel row of a 1-bit image.
pub const fn get_word_pitch(width: u32) -> u32 {
    (width + (BITMAP_WORD_BITS - 1) as u32) / BITMAP_WORD_BITS as u32
}

/// A 1-bit image.
#[derive(Clone, Default)]
pub struct Bitmap {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) words_per_row: u32,
    pub(crate) words: Vec<BitmapWord>,
}

impl Bitmap {
    /// Create a new blank bitmap of the given dimensions.
    pub fn new(width: u32, height: u32) -> Bitmap {
        let words_per_row = get_word_pitch(width);
        Bitmap {
            width,
            height,
            words_per_row,
            words: vec![0; (words_per_row * height) as usize],
        }
    }
    /// Create a new bitmap initialized with the given pixels. One byte = 8
    /// pixels, most significant bit on the left.
    pub fn from_bytes(width: u32, height: u32, bytes: &[u8]) -> Bitmap {
        let src_pitch = (width + 7) / 8;
        if bytes.len() != (src_pitch as usize * height as usize) {
            panic!("Bitmap::from_bytes(): input slice not exactly the right number of bytes");
        }
        let words_per_row = get_word_pitch(width);
        let mut bits = Vec::with_capacity((words_per_row * height) as usize);
        for row in bytes.chunks_exact(src_pitch as usize) {
            for group in row.chunks(BITMAP_WORD_BYTES) {
                bits.push(u32::from_be_bytes([
                    group.first().copied().unwrap_or(0),
                    group.get(1).copied().unwrap_or(0),
                    group.get(2).copied().unwrap_or(0),
                    group.get(3).copied().unwrap_or(0),
                ]))
            }
        }
        Bitmap {
            width,
            height,
            words_per_row,
            words: bits,
        }
    }
    /// Turn this bitmap into an array of byte pixels, most significant bit on
    /// the left. (If you are saving this into a Macintosh-originated format,
    /// be aware that *you* may have to pad the rows to a multiple of 2
    /// bytes.)
    pub fn to_bytes(&self) -> Vec<u8> {
        // This function isn't terribly efficient, but it is easy to understand.
        let out_rowbytes = ((self.width + 7) / 8) as usize;
        let mut ret = Vec::with_capacity(out_rowbytes * self.height as usize);
        for y in 0..self.height as usize {
            let src_row = &self.words[y * self.words_per_row as usize
                ..(y + 1) * self.words_per_row as usize];
            for (i, word) in src_row.iter().enumerate() {
                let in_x = i * BITMAP_WORD_BITS;
                let in_bytes = word.to_be_bytes();
                ret.push(in_bytes[0]);
                if in_x + 8 >= self.width as usize {
                    break;
                }
                ret.push(in_bytes[1]);
                if in_x + 16 >= self.width as usize {
                    break;
                }
                ret.push(in_bytes[2]);
                if in_x + 24 >= self.width as usize {
                    break;
                }
                ret.push(in_bytes[3]);
            }
        }
        ret
    }
    pub fn get_width(&self) -> u32 {
        self.width
    }
    pub fn get_height(&self) -> u32 {
        self.height
    }
    pub fn get_bounds(&self) -> Rectangle {
        Rectangle {
            left: 0,
            top: 0,
            right: self.width as i32,
            bottom: self.height as i32,
        }
    }
}
