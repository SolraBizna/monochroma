/**
 * This is a crate for processing and displaying monochrome bitmapped graphics.
 *
 * There are some optional features that are disabled by default:
 *
 * - `font`: Bitmapped font support. (Currently only Macintosh format)
 * - `netpbm`: netpbm image input and output (pbm, pgm, ppm).
 */
mod display;
#[doc(inline)]
pub use display::Display;

mod bitmap;
mod rectangle;
#[doc(inline)]
pub use bitmap::*;
#[doc(inline)]
pub use rectangle::*;
