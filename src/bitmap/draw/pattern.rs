use super::*;

/// A pattern where 50% of pixels are set.
pub const MEDIUM_GRAY: Pattern = Pattern {
    words: [
        0xAAAAAAAA, 0x55555555, 0xAAAAAAAA, 0x55555555, 0xAAAAAAAA,
        0x55555555, 0xAAAAAAAA, 0x55555555,
    ],
};
/// A pattern where 25% of pixels are set.
pub const LIGHT_GRAY: Pattern = Pattern {
    words: [
        0x88888888, 0x22222222, 0x88888888, 0x22222222, 0x88888888,
        0x22222222, 0x88888888, 0x22222222,
    ],
};
/// A pattern where 75% of pixels are set.
pub const DARK_GRAY: Pattern = Pattern {
    words: [
        0x77777777, 0xDDDDDDDD, 0x77777777, 0xDDDDDDDD, 0x77777777,
        0xDDDDDDDD, 0x77777777, 0xDDDDDDDD,
    ],
};
pub use DARK_GRAY as DARK_GREY;
pub use LIGHT_GRAY as LIGHT_GREY;
pub use MEDIUM_GRAY as MEDIUM_GREY;

/// A screen-aligned pattern of bits that can be applied to any draw mode.
/// Implemented by `()` (no pattern), `Pattern` (an 8x8 pattern), and whatever
/// else you want.
pub trait PatternTrait {
    /// Returns the pattern bits for the given word index `i` and row `y`.
    fn get_pattern_word(&self, i: u32, y: u32) -> BitmapWord;
}

impl<T: PatternTrait> PatternTrait for &T {
    fn get_pattern_word(&self, i: u32, y: u32) -> BitmapWord {
        (*self).get_pattern_word(i, y)
    }
}

impl PatternTrait for () {
    fn get_pattern_word(&self, _i: u32, _y: u32) -> BitmapWord {
        !0
    }
}

/// A screen-aligned 8x8 pattern that can be applied to any draw operation.
pub struct Pattern {
    words: [BitmapWord; 8],
}

impl Pattern {
    /// Create a pattern from an 8x8 bitmap composed of bytes.
    pub fn from_bytes(bytes: &[u8; 8]) -> Pattern {
        Pattern {
            words: bytes.map(|x| BitmapWord::from_ne_bytes([x, x, x, x])),
        }
    }
    /// Return this pattern as a 8x8 bitmap composed of bytes.
    pub fn to_bytes(&self) -> [u8; 8] {
        self.words.map(|x| x as u8)
    }
}

impl PatternTrait for Pattern {
    fn get_pattern_word(&self, _i: u32, y: u32) -> BitmapWord {
        self.words[(y % 8) as usize]
    }
}
