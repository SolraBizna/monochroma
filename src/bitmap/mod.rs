use super::*;

mod draw;
mod pbm;

pub use draw::*;

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
    pub(crate) pitch_words: u32,
    pub(crate) bits: Vec<BitmapWord>,
}

impl Bitmap {
    /// Create a new blank bitmap of the given dimensions.
    pub fn new(width: u32, height: u32) -> Bitmap {
        let pitch_words = get_word_pitch(width);
        Bitmap {
            width,
            height,
            pitch_words,
            bits: vec![0; (pitch_words * height) as usize],
        }
    }
    /// Create a new bitmap initialized with the given pixels. One byte = 8
    /// pixels, most significant bit on the left.
    pub fn from_bytes(width: u32, height: u32, bytes: &[u8]) -> Bitmap {
        let src_pitch = (width + 7) / 8;
        if bytes.len() != (src_pitch as usize * height as usize) {
            panic!("Bitmap::from_bytes(): input slice not exactly the right nubmer of bytes");
        }
        let pitch_words = get_word_pitch(width);
        let mut bits = Vec::with_capacity((pitch_words * height) as usize);
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
            pitch_words,
            bits,
        }
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
