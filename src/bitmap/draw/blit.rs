use std::cmp::Ordering;

use super::*;

impl Bitmap {
    /// Blit the given rectangle of pixels from the source into ourselves, with
    /// the top-left corner of the rectangle being placed at the given x and y
    /// coordinates.
    pub fn blit_bits<Mode: TransferMode>(
        &mut self,
        mode: Mode,
        clip_rect: Option<Rectangle>,
        src: &Bitmap,
        src_rect: Option<Rectangle>,
        mut dst_x: i32,
        mut dst_y: i32,
    ) {
        let clip_rect = clip_rect
            .map(|x| x.intersection(self.get_bounds()))
            .unwrap_or(self.get_bounds());
        let mut src_rect = src_rect.unwrap_or(src.get_bounds());
        if dst_x < clip_rect.left {
            src_rect.left -= dst_x - clip_rect.left;
            dst_x = clip_rect.left;
        }
        if src_rect.left < 0 {
            dst_x -= src_rect.left;
            src_rect.left = 0;
        }
        if dst_y < clip_rect.top {
            src_rect.top -= dst_y - clip_rect.top;
            dst_y = clip_rect.top;
        }
        if src_rect.top < 0 {
            dst_y -= src_rect.top;
            src_rect.top = 0;
        }
        if dst_x >= clip_rect.get_width() as i32 {
            return;
        }
        if dst_y >= clip_rect.get_height() as i32 {
            return;
        }
        let mut src_rect = src_rect.intersection(src.get_bounds());
        if let Some(overshoot) = src_rect
            .get_width()
            .checked_sub(clip_rect.right as u32 - dst_x as u32)
        {
            src_rect.right -= overshoot as i32;
        }
        if let Some(overshoot) = src_rect
            .get_height()
            .checked_sub(clip_rect.right as u32 - dst_y as u32)
        {
            src_rect.bottom -= overshoot as i32;
        }
        if src_rect.is_empty() {
            return;
        }
        let src_left = src_rect.left as u32;
        let src_top = src_rect.top as u32;
        let src_right = src_rect.right as u32;
        let dst_left = dst_x as u32;
        let dst_top = dst_y as u32;
        let dst_right = dst_left + src_rect.get_width();
        let dst_bottom = dst_top + src_rect.get_height();
        let (in_start_word, in_stop_word, _, _) =
            calculate_span_mask(src_left, src_right);
        let in_word_count = (in_stop_word + 1) - in_start_word;
        let i = in_start_word + src_top * src.pitch_words;
        let src_bit_align = src_left % BITMAP_WORD_BITS as u32;
        let dst_bit_align = dst_left % BITMAP_WORD_BITS as u32;
        match src_bit_align.cmp(&dst_bit_align) {
            Ordering::Less => {
                // We must shift to the RIGHT
                let slip = dst_bit_align - src_bit_align;
                inner_blit(
                    mode,
                    (i..(i + src_rect.get_height() * src.pitch_words))
                        .step_by(src.pitch_words as usize)
                        .map(|offset| {
                            let words = &src.bits[offset as usize
                                ..(offset + in_word_count) as usize];
                            Biterator::new(words, slip, 0)
                        }),
                    self,
                    dst_left,
                    dst_top,
                    dst_right,
                    dst_bottom,
                )
            }
            Ordering::Equal => {
                // No shifting required
                inner_blit(
                    mode,
                    (i..(i + src_rect.get_height() * src.pitch_words))
                        .step_by(src.pitch_words as usize)
                        .map(|offset| {
                            src.bits[offset as usize
                                ..(offset + in_word_count) as usize]
                                .iter()
                                .copied()
                        }),
                    self,
                    dst_left,
                    dst_top,
                    dst_right,
                    dst_bottom,
                )
            }
            Ordering::Greater => {
                // We must shift to the LEFT
                let slip = src_bit_align - dst_bit_align;
                inner_blit(
                    mode,
                    (i..(i + src_rect.get_height() * src.pitch_words))
                        .step_by(src.pitch_words as usize)
                        .map(|offset| {
                            let words = &src.bits[offset as usize
                                ..(offset + in_word_count) as usize];
                            Biterator::new(words, 0, slip)
                        }),
                    self,
                    dst_left,
                    dst_top,
                    dst_right,
                    dst_bottom,
                )
            }
        }
    }
}

fn inner_blit<Mode: TransferMode>(
    mode: Mode,
    mut src_rows: impl Iterator<Item = impl Iterator<Item = BitmapWord>>,
    dst: &mut Bitmap,
    dst_left: u32,
    dst_top: u32,
    dst_right: u32,
    dst_bottom: u32,
) {
    let (out_start_word, out_stop_word, left_mask, right_mask) =
        calculate_span_mask(dst_left, dst_right);
    let mut i = (out_start_word + dst_top * dst.pitch_words) as usize;
    if out_start_word == out_stop_word {
        let combined_mask = left_mask & right_mask;
        for y in dst_top..dst_bottom {
            let mut src_row = src_rows.next().unwrap();
            dst.bits[i] = dst.bits[i] & !combined_mask
                | (mode.combine(
                    src_row.next().unwrap(),
                    dst.bits[i],
                    out_start_word,
                    y,
                ) & combined_mask);
            i += dst.pitch_words as usize;
        }
        debug_assert!(src_rows.next().is_none())
    } else {
        let out_stride =
            (dst.pitch_words - (out_stop_word - out_start_word)) as usize;
        for y in dst_top..dst_bottom {
            let mut src_row = src_rows.next().unwrap();
            dst.bits[i] = dst.bits[i] & !left_mask
                | (mode.combine(
                    src_row.next().unwrap(),
                    dst.bits[i],
                    out_start_word,
                    y,
                ) & left_mask);
            i += 1;
            for x in out_start_word + 1..out_stop_word {
                dst.bits[i] =
                    mode.combine(src_row.next().unwrap(), dst.bits[i], x, y);
                i += 1;
            }
            dst.bits[i] = dst.bits[i] & !right_mask
                | (mode.combine(
                    src_row.next().unwrap(),
                    dst.bits[i],
                    out_stop_word,
                    y,
                ) & right_mask);
            i += out_stride;
        }
        debug_assert!(src_rows.next().is_none())
    }
}

struct Biterator<'a> {
    /// The next bits to yield
    bits: BitmapWord,
    /// The bits into which we will shove the next word from rem
    overflowed_bits: BitmapWord,
    rem: &'a [BitmapWord],
    slip: u32,
    owari: bool,
    owatta: bool,
}

impl Biterator<'_> {
    pub fn new(
        slice: &[BitmapWord],
        zeroes_on_left: u32,
        bits_to_eat: u32,
    ) -> Biterator {
        debug_assert!(bits_to_eat < 32);
        debug_assert!(zeroes_on_left < 32);
        debug_assert!(!slice.is_empty());
        debug_assert_ne!(zeroes_on_left, bits_to_eat);
        let mut huge = (slice[0] as u64)
            << (bits_to_eat + BITMAP_WORD_BITS as u32)
            >> zeroes_on_left;
        let rem;
        let slip;
        let owari;
        if zeroes_on_left < bits_to_eat {
            // we have to eat an extra word
            slip = bits_to_eat - zeroes_on_left;
            if slice.len() > 1 {
                huge |= (slice[1] as u64) << bits_to_eat >> zeroes_on_left;
                rem = &slice[2..];
                owari = false;
            } else {
                rem = &[];
                owari = true;
            }
        } else {
            // no need to eat an extra word
            rem = &slice[1..];
            slip = bits_to_eat + BITMAP_WORD_BITS as u32 - zeroes_on_left;
            owari = false;
        }
        let bits = (huge >> 32) as BitmapWord;
        let overflowed_bits = huge as BitmapWord;
        Biterator {
            bits,
            overflowed_bits,
            rem,
            slip,
            owari,
            owatta: false,
        }
    }
}

impl Iterator for Biterator<'_> {
    type Item = BitmapWord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.owatta {
            None
        } else if self.owari {
            self.owatta = true;
            Some(self.bits)
        } else {
            let ret = self.bits;
            self.bits = self.overflowed_bits;
            let tsugi;
            if self.rem.is_empty() {
                self.owari = true;
                tsugi = 0;
            } else {
                tsugi = self.rem[0];
                self.rem = &self.rem[1..];
            };
            self.overflowed_bits = tsugi << self.slip;
            self.bits |= tsugi >> (BITMAP_WORD_BITS as u32 - self.slip);
            Some(ret)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn basic_biterator() {
        let in_bits = [0x01234567, 0x89ABCDEF];
        assert_eq!(
            Biterator::new(&in_bits, 0, 4).collect::<Vec<u32>>(),
            &[0x12345678, 0x9ABCDEF0],
        );
        assert_eq!(
            Biterator::new(&in_bits, 4, 0).collect::<Vec<u32>>(),
            &[0x00123456, 0x789ABCDE, 0xF0000000],
        );
        assert_eq!(
            Biterator::new(&in_bits, 4, 8).collect::<Vec<u32>>(),
            &[0x02345678, 0x9ABCDEF0],
        );
    }
}
