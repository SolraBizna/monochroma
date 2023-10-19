use super::*;

mod pattern;
pub use pattern::*;
mod transfer;
pub use transfer::*;

mod blit;
mod shapes;

fn calculate_span_mask(left: u32, right: u32) -> (u32, u32, u32, u32) {
    let start_word = left / BITMAP_WORD_BITS as u32;
    let stop_word = (right - 1) / BITMAP_WORD_BITS as u32;
    let left_mask = !0 >> (left % BITMAP_WORD_BITS as u32);
    let right_mask = if right % BITMAP_WORD_BITS as u32 == 0 {
        !0
    } else {
        !0 << (BITMAP_WORD_BITS as u32 - right % BITMAP_WORD_BITS as u32)
    };
    debug_assert!(left_mask != 0);
    debug_assert!(right_mask != 0);
    (start_word, stop_word, left_mask, right_mask)
}
