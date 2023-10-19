use super::*;

impl Bitmap {
    /// Fill in all pixels of the given rectangle.
    pub fn fill_rect<Mode>(
        &mut self,
        mode: Mode,
        clip_rect: Option<Rectangle>,
        rectangle: Rectangle,
    ) where
        Mode: TransferMode,
    {
        let clip_rect = clip_rect
            .map(|x| x.intersection(self.get_bounds()))
            .unwrap_or(self.get_bounds());
        let rectangle = rectangle.intersection(clip_rect);
        if rectangle.is_empty() {
            return;
        }
        let left = rectangle.left as u32;
        let right = rectangle.right as u32;
        let top = rectangle.top as u32;
        let bottom = rectangle.bottom as u32;
        let (start_word, stop_word, left_mask, right_mask) =
            calculate_span_mask(left, right);
        let mut i = (start_word + top * self.pitch_words) as usize;
        if start_word == stop_word {
            let combined_mask = left_mask & right_mask;
            for y in top..bottom {
                self.bits[i] = self.bits[i] & !combined_mask
                    | (mode.combine(
                        combined_mask,
                        self.bits[i],
                        start_word,
                        y,
                    ) & combined_mask);
                i += self.pitch_words as usize;
            }
        } else {
            let stride =
                (self.pitch_words - (stop_word - start_word)) as usize;
            for y in top..bottom {
                self.bits[i] = self.bits[i] & !left_mask
                    | (mode.combine(left_mask, self.bits[i], start_word, y)
                        & left_mask);
                i += 1;
                for x in start_word + 1..stop_word {
                    self.bits[i] = mode.combine(!0, self.bits[i], x, y);
                    i += 1;
                }
                self.bits[i] = self.bits[i] & !right_mask
                    | (mode.combine(right_mask, self.bits[i], stop_word, y)
                        & right_mask);
                i += stride;
            }
        }
    }
    /// Draw a line border around the given rectangle. The drawn pixels will be
    /// strictly inside the given rectangle.
    pub fn stroke_rect<Mode>(
        &mut self,
        mode: Mode,
        clip_rect: Option<Rectangle>,
        rectangle: Rectangle,
        x_thickness: u32,
        y_thickness: u32,
    ) where
        Mode: TransferMode,
    {
        if rectangle.get_width() <= x_thickness * 2
            || rectangle.get_height() <= y_thickness * 2
        {
            self.fill_rect(mode, clip_rect, rectangle);
        } else {
            self.fill_rect(
                &mode,
                clip_rect,
                Rectangle {
                    left: rectangle.left,
                    right: rectangle.right,
                    top: rectangle.top,
                    bottom: rectangle.top + y_thickness as i32,
                },
            );
            self.fill_rect(
                &mode,
                clip_rect,
                Rectangle {
                    left: rectangle.left,
                    right: rectangle.left + x_thickness as i32,
                    top: rectangle.top + y_thickness as i32,
                    bottom: rectangle.bottom - y_thickness as i32,
                },
            );
            self.fill_rect(
                &mode,
                clip_rect,
                Rectangle {
                    left: rectangle.right - x_thickness as i32,
                    right: rectangle.right,
                    top: rectangle.top + y_thickness as i32,
                    bottom: rectangle.bottom - y_thickness as i32,
                },
            );
            self.fill_rect(
                &mode,
                clip_rect,
                Rectangle {
                    left: rectangle.left,
                    right: rectangle.right,
                    top: rectangle.bottom - y_thickness as i32,
                    bottom: rectangle.bottom,
                },
            );
        }
    }
}
