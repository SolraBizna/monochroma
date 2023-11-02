use super::*;

use std::{io::Read, ops::RangeInclusive, sync::OnceLock};

use anyhow::{anyhow, Context};

/// A loaded bitmap font.
pub struct Font {
    glyph_range: RangeInclusive<u16>,
    /// bits!
    bitmap: Bitmap,
    /// number of pixels between top of topmost bit and baseline
    ascent: i16,
    /// number of pixels between baseline and bottom of bottommost bit
    descent: i16,
    /// extra pixels between lines
    leading: i16,
    /// X coordinates of left edges of glyphs
    glyph_locations: Vec<u16>,
    /// Offsets and advances of glyphs
    glyph_offsetwidths: Vec<(i8, u8)>,
    /// Cached width of space character
    space_width: OnceLock<u8>,
}

impl Font {
    /// Attempts to parse the given input as a Macintosh Toolbox FONT/NFNT.
    pub fn read_mac_font(mut i: impl Read) -> anyhow::Result<Font> {
        let mut header_buf = [0; 26];
        i.read_exact(&mut header_buf)
            .context("Error reading NFNT header.")?;
        if u16::from_be_bytes([header_buf[0], header_buf[1]]) & 0x280 != 0 {
            return Err(anyhow!("Color NFNTs are not supported."));
        }
        let first_glyph = u16::from_be_bytes([header_buf[2], header_buf[3]]);
        let last_glyph = u16::from_be_bytes([header_buf[4], header_buf[5]]);
        if last_glyph < first_glyph {
            return Err(anyhow!(
                "last_glyph is before first_glyph (nonsense!)"
            ));
        } else if last_glyph == 0xFFFF {
            return Err(anyhow!(
                "font didn't leave room for the fallback glyph"
            ));
        }
        let font_height = u16::from_be_bytes([header_buf[14], header_buf[15]]);
        let ascent = i16::from_be_bytes([header_buf[18], header_buf[19]]);
        let descent = i16::from_be_bytes([header_buf[20], header_buf[21]]);
        let leading = i16::from_be_bytes([header_buf[22], header_buf[23]]);
        let rowshorts = u16::from_be_bytes([header_buf[24], header_buf[25]]);
        let mut buf = vec![0; rowshorts as usize * 2 * font_height as usize];
        i.read_exact(&mut buf)
            .context("Error reading NFNT bitmap data.")?;
        let bitmap = Bitmap::from_bytes(
            rowshorts as u32 * 16,
            font_height as u32,
            &buf,
        );
        let num_glyphs = last_glyph as usize - first_glyph as usize + 2;
        let num_locations = num_glyphs + 1;
        buf.resize((num_locations + num_glyphs) * 2, 0);
        i.read_exact(&mut buf)
            .context("Error reading NFNT glyph location data.")?;
        let mut glyph_locations = Vec::with_capacity(num_locations);
        for i in (0..num_locations * 2).step_by(2) {
            glyph_locations.push(u16::from_be_bytes([buf[i], buf[i + 1]]));
        }
        if glyph_locations.windows(2).any(|w| w[0] > w[1]) {
            return Err(anyhow!("Glyph locations in NFNT were not in ascending order. The font is corrupted."));
        }
        if *glyph_locations.last().unwrap() as u32 > bitmap.get_width() {
            return Err(anyhow!("Glyph locations in NFNT extend past the right edge of the bitmap. The font is corrupted."));
        }
        let mut glyph_offsetwidths = Vec::with_capacity(num_glyphs);
        for i in (num_locations * 2..buf.len()).step_by(2) {
            glyph_offsetwidths.push((buf[i] as i8, buf[i + 1]));
        }
        Ok(Font {
            glyph_range: first_glyph..=last_glyph,
            bitmap,
            ascent,
            descent,
            leading,
            glyph_locations,
            glyph_offsetwidths,
            space_width: OnceLock::new(),
        })
    }
    /// Get the number of pixels that are above the baseline.
    pub fn get_ascent(&self) -> i32 {
        self.ascent as i32
    }
    /// Get the number of pixels that are below the baseline.
    pub fn get_descent(&self) -> i32 {
        self.descent as i32
    }
    /// Get the number of pixels that should be added/removed between
    /// lines (in addition to the ascent and descent).
    pub fn get_leading(&self) -> i32 {
        self.leading as i32
    }
    /// Get a reference to the entire font bitmap.
    pub fn get_bitmap(&self) -> &Bitmap {
        &self.bitmap
    }
    /// Gets enough information to render a given glyph in this font.
    /// Returns:
    /// - The rectangle within the font bitmap that contains this glyph's
    ///   pixels.
    /// - The amount to add to the pen X coordinate to get the left edge
    ///   of the glyph bitmap.
    /// - The amount to advance the pen X coordinate by, where the next
    ///   character should begin.
    /// - True if the glyph actually exists in the font. (If false, the
    ///   returned information is for the fallback glyph.)
    pub fn get_glyph(&self, glyph_id: u16) -> (Rectangle, i32, u32, bool) {
        let (glyph_index, present);
        if self.glyph_range.contains(&glyph_id) {
            glyph_index = (glyph_id - self.glyph_range.start()) as usize;
            present = true;
        } else {
            glyph_index = self.glyph_range.clone().count();
            present = false;
        };
        let (offset, advance) = self.glyph_offsetwidths[glyph_index];
        if (offset, advance) == (-1, 255) {
            return self.get_glyph(!0);
        }
        let left = self.glyph_locations[glyph_index] as i32;
        let right = self.glyph_locations[glyph_index + 1] as i32;
        (
            Rectangle {
                left,
                top: 0,
                right,
                bottom: self.bitmap.get_height() as i32,
            },
            offset as i32,
            advance as u32,
            present,
        )
    }
    /// Makes a version of this font that is bolder.
    ///
    /// - Every glyph is made to advance by one more pixel.
    /// - For every marked pixel in a glyph, its neighbor to the right is also
    ///   marked. (A one-pixel stem becomes two, a two-pixel stem becomes
    ///   three, etc.)
    pub fn make_bold(&self) -> Font {
        let mut new_glyph_locations =
            Vec::with_capacity(self.glyph_locations.len());
        new_glyph_locations.push(0);
        let mut x = 0;
        for w in self.glyph_locations.windows(2) {
            let (start, stop) = (w[0], w[1]);
            if start != stop {
                x = x + (stop - start) + 1;
            }
            new_glyph_locations.push(x);
        }
        let mut new_bitmap = Bitmap::new(
            *new_glyph_locations.last().unwrap() as u32,
            self.bitmap.height,
        );
        let mut dst_x = 0;
        for w in self.glyph_locations.windows(2) {
            let (start, stop) = (w[0], w[1]);
            if start != stop {
                let dst_left = dst_x;
                dst_x += stop - start + 1;
                new_bitmap.blit_bits(
                    ModeCopy(()),
                    None,
                    &self.bitmap,
                    Some(Rectangle {
                        left: start as i32,
                        right: stop as i32,
                        ..self.bitmap.get_bounds()
                    }),
                    dst_left as i32,
                    0,
                );
                new_bitmap.blit_bits(
                    ModeOr(()),
                    None,
                    &self.bitmap,
                    Some(Rectangle {
                        left: start as i32,
                        right: stop as i32,
                        ..self.bitmap.get_bounds()
                    }),
                    dst_left as i32 + 1,
                    0,
                );
            }
        }
        Font {
            glyph_range: self.glyph_range.clone(),
            bitmap: new_bitmap,
            ascent: self.ascent,
            descent: self.descent,
            leading: self.leading,
            glyph_locations: new_glyph_locations,
            glyph_offsetwidths: self
                .glyph_offsetwidths
                .iter()
                .map(|(offset, advance)| (*offset, advance.saturating_add(1)))
                .collect(),
            space_width: OnceLock::new(),
        }
    }
    /// Makes a version of this font that is italic. (If it is also to be made
    /// bold, that should be done first.)
    ///
    /// - Each two rows is offset one pixel to the left of the two rows above
    ///   it.
    pub fn make_italic(&self) -> Font {
        let total_height = self.get_ascent() + self.get_descent();
        let (num_steps, offset_offset) = if total_height & 1 == 0 {
            (total_height / 2, 0)
        } else {
            (((total_height + 1) / 2), 1)
        };
        let mut new_glyph_locations =
            Vec::with_capacity(self.glyph_locations.len());
        new_glyph_locations.push(0);
        let mut x = 0;
        for w in self.glyph_locations.windows(2) {
            let (start, stop) = (w[0], w[1]);
            if start != stop {
                x += (stop - start) + (num_steps - 1).max(0) as u16;
            }
            new_glyph_locations.push(x);
        }
        let mut new_bitmap = Bitmap::new(
            *new_glyph_locations.last().unwrap() as u32,
            self.bitmap.height,
        );
        let mut dst_x = 0;
        for w in self.glyph_locations.windows(2) {
            let (start, stop) = (w[0], w[1]);
            if start != stop {
                let dst_left = dst_x;
                dst_x += (stop - start) + (num_steps - 1).max(0) as u16;
                for offset in 0..num_steps {
                    let bottom = (num_steps - offset) * 2 - offset_offset;
                    new_bitmap.blit_bits(
                        ModeCopy(()),
                        None,
                        &self.bitmap,
                        Some(Rectangle {
                            left: start as i32,
                            right: stop as i32,
                            top: bottom - 2,
                            bottom,
                        }),
                        dst_left as i32 + offset,
                        bottom - 2,
                    );
                }
            }
        }
        Font {
            glyph_range: self.glyph_range.clone(),
            bitmap: new_bitmap,
            ascent: self.ascent,
            descent: self.descent,
            leading: self.leading,
            glyph_locations: new_glyph_locations,
            glyph_offsetwidths: self.glyph_offsetwidths.clone(),
            space_width: OnceLock::new(),
        }
    }
    /// Makes a version of this font that is underlined. (If it is also to be
    /// made bold and/or italic, those should be done first.)
    ///
    /// - If the glyph bitmap is not wide enough to contain both its start and
    ///   end pen points, it is widened accordingly.
    /// - A one-pixel thick horizontal line is drawn on the second row below
    ///   the baseline, starting at the start pen point, and ending at the end
    ///   pen point.
    /// - Any part of this line which has a set pixel on any of its 8 neighbors
    ///   is erased.
    pub fn make_underline(&self) -> Font {
        let mut new_glyph_locations =
            Vec::with_capacity(self.glyph_locations.len());
        let mut new_glyph_offsetwidths =
            Vec::with_capacity(self.glyph_offsetwidths.len());
        new_glyph_locations.push(0);
        let mut x = 0;
        for (w, (offset, advance)) in self
            .glyph_locations
            .windows(2)
            .zip(self.glyph_offsetwidths.iter())
        {
            if (*offset, *advance) == (!0, !0) {
                new_glyph_locations.push(x);
                new_glyph_offsetwidths.push((*offset, *advance));
                continue;
            }
            let (mut offset, advance) = (*offset, *advance);
            let (start, stop) = (w[0], w[1]);
            let mut width = stop - start;
            if offset > 0 {
                // extend bitmap to left
                width += offset as u16;
                offset = 0;
            }
            if (advance as u16) > width {
                // extend bitmap to right
                let shortfall = advance as u16 - width;
                width += shortfall;
            }
            x += width;
            new_glyph_locations.push(x);
            new_glyph_offsetwidths.push((offset, advance));
        }
        let new_descent = self.descent.max(2);
        let new_height = self
            .bitmap
            .get_height()
            .max(self.ascent as u32 + new_descent as u32);
        let mut new_bitmap = Bitmap::new(
            *new_glyph_locations.last().unwrap() as u32,
            new_height,
        );
        let mut dst_x = 0;
        for (w, (offset, advance)) in self
            .glyph_locations
            .windows(2)
            .zip(self.glyph_offsetwidths.iter())
        {
            if (*offset, *advance) == (!0, !0) {
                continue;
            }
            let (start, stop) = (w[0], w[1]);
            let mut width = stop - start;
            let (offset, advance) = (*offset, *advance);
            let mut dst_left = dst_x;
            let mut underline_left = dst_x;
            if offset > 0 {
                // extend bitmap to left
                width += offset as u16;
                dst_left += offset as i32;
            } else {
                // push underline to right
                underline_left -= offset as i32;
            }
            if (advance as u16) > width {
                // extend bitmap to right
                let shortfall = advance as u16 - width;
                width += shortfall;
            }
            let underline_right = underline_left + advance as i32;
            new_bitmap.fill_rect(
                ModeCopy(()),
                None,
                Rectangle {
                    left: underline_left,
                    right: underline_right,
                    top: self.ascent as i32 + 1,
                    bottom: self.ascent as i32 + 2,
                },
            );
            for dy in [-1, 0, 1] {
                for dx in [-1, 0, 1] {
                    if dx == dy {
                        continue;
                    }
                    new_bitmap.blit_bits(
                        ModeInverseAnd(()),
                        Some(Rectangle {
                            left: dst_x,
                            right: dst_x + width as i32,
                            ..new_bitmap.get_bounds()
                        }),
                        &self.bitmap,
                        Some(Rectangle {
                            left: start as i32,
                            right: stop as i32,
                            ..self.bitmap.get_bounds()
                        }),
                        dst_left + dx,
                        dy,
                    );
                }
            }
            new_bitmap.blit_bits(
                ModeOr(()),
                None,
                &self.bitmap,
                Some(Rectangle {
                    left: start as i32,
                    right: stop as i32,
                    ..self.bitmap.get_bounds()
                }),
                dst_left,
                0,
            );
            dst_x += width as i32;
        }
        Font {
            glyph_range: self.glyph_range.clone(),
            bitmap: new_bitmap,
            ascent: self.ascent,
            descent: new_descent,
            leading: self.leading,
            glyph_locations: new_glyph_locations,
            glyph_offsetwidths: new_glyph_offsetwidths,
            space_width: OnceLock::new(),
        }
    }
    /// Measure text, as though drawn with `Bitmap::draw_text` but without
    /// doing any drawing.
    pub fn measure_text<'a>(
        mut pen_x: i32,
        mut pen_y: i32,
        fonts: impl Fn(usize) -> Option<&'a Font>,
        elements: impl Iterator<Item = TextElement>,
    ) -> TextMeasurements {
        let pen_start = (pen_x, pen_y);
        let mut drawn_rectangle = Rectangle::default();
        for element in elements {
            match element {
                TextElement::DrawGlyph(glyph) => {
                    let (font, rect, offset, advance, _present) =
                        lookup_glyph(&fonts, glyph);
                    let (draw_x, draw_y) =
                        (pen_x + offset, pen_y - font.get_ascent());
                    drawn_rectangle = drawn_rectangle.union(Rectangle {
                        left: draw_x,
                        right: draw_x + rect.get_width() as i32,
                        top: draw_y,
                        bottom: draw_y + rect.get_height() as i32,
                    });
                    pen_x += advance as i32;
                }
                TextElement::MovePen(new_x, new_y) => {
                    (pen_x, pen_y) = (new_x, new_y);
                }
                TextElement::AdvancePen(advance) => {
                    pen_x = pen_x.saturating_add(advance);
                }
            }
        }
        TextMeasurements {
            drawn_rectangle,
            pen_start,
            pen_end: (pen_x, pen_y),
        }
    }
    /// Measure a single glyph. You can use this function to somewhat
    /// inefficiently implement `measure_text`, but it is more useful when
    /// building data structures related to rendering, selection, editing...
    pub fn measure_glyph<'a>(
        fonts: impl Fn(usize) -> Option<&'a Font>,
        glyph: u16,
    ) -> GlyphMeasurement {
        let (font, rect, offset, advance, present) =
            lookup_glyph(fonts, glyph);
        GlyphMeasurement {
            drawn_rectangle: Rectangle {
                left: offset,
                right: offset + rect.get_width() as i32,
                top: -font.get_ascent(),
                bottom: -font.get_ascent() + rect.get_height() as i32,
            },
            advance,
            present,
        }
    }
    /// Returns the "advance" measurement for the space glyph. This is the only
    /// part of this code that assumes a particular glyph has a particular
    /// purpose. It assumes the space glyph is glyph 0x20 (ASCII space). You
    /// could obtain this value yourself by calling `measure_glyph`, but this
    /// value is often needed if it's needed at all, and this method caches the
    /// value after the first call.
    pub fn get_space_width(&self) -> u32 {
        if let Some(space_width) = self.space_width.get() {
            *space_width as u32
        } else {
            let ret =
                Font::measure_glyph(|n| [self].get(n).copied(), 0x20).advance;
            // It's harmless if we attempt to set it more than once.
            let _ = self.space_width.set(ret.try_into().expect("Internal error: advance no longer fits into a u8, this part of get_space_width needs to be updated!"));
            ret as u32
        }
    }
}

/// Commands for `draw_text` and `measure_text`.
#[derive(Clone, Debug)]
pub enum TextElement {
    /// Draw the given glyph at the current pen coordinates, and advance the
    /// pen by the appropriate amount.
    ///
    /// Glyphs are given as code points in whatever encoding your font expects.
    /// 0xFFFF is guaranteed never to be present in an NFNT, so you can use it
    /// to display a non-encodable character.
    DrawGlyph(u16),
    /// Move the pen to the given absolute coordinates.
    MovePen(i32, i32),
    /// Advance the pen by the given number of pixels, as if we rendered a
    /// glyph with this amount of advance.
    AdvancePen(i32),
}

impl Bitmap {
    pub fn draw_text<'a, Mode: TransferMode>(
        &mut self,
        mode: Mode,
        clip_rect: Option<Rectangle>,
        mut pen_x: i32,
        mut pen_y: i32,
        fonts: impl Fn(usize) -> Option<&'a Font>,
        elements: impl Iterator<Item = TextElement>,
    ) -> (i32, i32) {
        for element in elements {
            match element {
                TextElement::DrawGlyph(glyph) => {
                    let (font, rect, offset, advance, _present) =
                        lookup_glyph(&fonts, glyph);
                    let (draw_x, draw_y) =
                        (pen_x + offset, pen_y - font.get_ascent());
                    self.blit_bits(
                        &mode,
                        clip_rect,
                        font.get_bitmap(),
                        Some(rect),
                        draw_x,
                        draw_y,
                    );
                    pen_x += advance as i32;
                }
                TextElement::MovePen(new_x, new_y) => {
                    (pen_x, pen_y) = (new_x, new_y);
                }
                TextElement::AdvancePen(advance) => {
                    pen_x = pen_x.saturating_add(advance);
                }
            }
        }
        (pen_x, pen_y)
    }
}

pub struct TextMeasurements {
    pub drawn_rectangle: Rectangle,
    pub pen_start: (i32, i32),
    pub pen_end: (i32, i32),
}

pub struct GlyphMeasurement {
    /// The rectangle of touched bits, where (0, 0) is the pen location.
    pub drawn_rectangle: Rectangle,
    /// The amount to add to the pen X coordinate after drawing.
    pub advance: u32,
    /// True if the glyph was found, false if this is a fallback glyph.
    pub present: bool,
}

fn lookup_glyph<'a>(
    fonts: impl Fn(usize) -> Option<&'a Font>,
    glyph: u16,
) -> (&'a Font, Rectangle, i32, u32, bool) {
    let first_font = fonts(0).expect("At least one font must be provided!");
    let (mut font, (mut rect, mut offset, mut advance, mut present)) =
        (first_font, first_font.get_glyph(glyph));
    if !present {
        let (missing_rect, missing_offset, missing_advance) =
            (rect, offset, advance);
        for n in 1.. {
            let Some(candidate) = fonts(n) else { break };
            (font, (rect, offset, advance, present)) =
                (candidate, candidate.get_glyph(glyph));
            if present {
                break;
            }
        }
        if !present {
            // use the missing glyph from the first font if
            // none had it
            (rect, offset, advance) =
                (missing_rect, missing_offset, missing_advance);
        }
    }
    (font, rect, offset, advance, present)
}
