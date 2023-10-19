use super::*;

/// Any means of combining source and destination bits.
///
/// Implemented by the `Mode` types:
/// - [`ModeCopy`](struct.ModeCopy.html) = `new`
/// - [`ModeInverseCopy`](struct.ModeInverseCopy.html) = `!new`
/// - [`ModeAnd`](struct.ModeAnd.html) = `new & existing`
/// - [`ModeInverseAnd`](struct.ModeInverseAnd.html) = `!new & existing`
///   (also known as "bit clear")
/// - [`ModeOr`](struct.ModeOr.html) = `new | existing`
/// - [`ModeInverseOr`](struct.ModeInverseOr.html) = `!new | existing`
/// - [`ModeXor`](struct.ModeXor.html) = `new ^ existing`
/// - [`ModeInverseXor`](struct.ModeInverseXor.html) = `!new ^ existing`
/// You can specify a `Pattern` to combine with the bits, or `()`` to use the
/// bits directly.
pub trait TransferMode {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        word_index: u32,
        y: u32,
    ) -> BitmapWord;
}

impl<T: TransferMode> TransferMode for &T {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        word_index: u32,
        y: u32,
    ) -> BitmapWord {
        (*self).combine(new, existing, word_index, y)
    }
}

/// Simply overwrite the existing bits.
pub struct ModeCopy<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeCopy<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        _existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        new & self.0.get_pattern_word(i, y)
    }
}

/// Simply overwrite the existing bits with the *inversion* of the new bits.
pub struct ModeInverseCopy<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeInverseCopy<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        _existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        !(new & self.0.get_pattern_word(i, y))
    }
}

/// Bitwise AND. Results in a set pixel when the new AND existing pixel are
/// set.
pub struct ModeAnd<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeAnd<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        (new & self.0.get_pattern_word(i, y)) & existing
    }
}

/// Bitwise AND with inverted input. Results in a set pixel when the new pixel
/// is clear AND the existing pixel is set. (Corresponds to the "Bit Clear"
/// transfer mode from QuickDraw.)

pub struct ModeInverseAnd<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeInverseAnd<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        !(new & self.0.get_pattern_word(i, y)) & existing
    }
}

/// Bitwise OR. Results in a set pixel when the new OR existing pixel are
/// set.
pub struct ModeOr<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeOr<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        (new & self.0.get_pattern_word(i, y)) | existing
    }
}

/// Bitwise OR with inverted input. Results in a set pixel when the new pixel
/// is clear OR the existing existing pixel is set.
pub struct ModeInverseOr<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeInverseOr<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        !(new & self.0.get_pattern_word(i, y)) | existing
    }
}

/// Bitwise XOR. Results in a set pixel when the new and existing pixels are
/// different.
pub struct ModeXor<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeXor<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        (new & self.0.get_pattern_word(i, y)) ^ existing
    }
}

/// Bitwise XOR with inverted input. Results in a set pixel when the new and
/// existing pixels are the same.
pub struct ModeInverseXor<Pattern: PatternTrait>(pub Pattern);
impl<Pattern: PatternTrait> TransferMode for ModeInverseXor<Pattern> {
    fn combine(
        &self,
        new: BitmapWord,
        existing: BitmapWord,
        i: u32,
        y: u32,
    ) -> BitmapWord {
        !(new & self.0.get_pattern_word(i, y)) ^ existing
    }
}
