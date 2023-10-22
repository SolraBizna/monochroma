/**
 * This is a crate for processing and displaying monochrome bitmapped graphics.
 *
 * There are some optional features that are disabled by default:
 *
 * - `display`: Display support, using SDL2 and OpenGL 3.1.
 * - `font`: Bitmapped font support. (Currently only Macintosh format)
 * - `netpbm`: netpbm image input and output (pbm, pgm, ppm).
 */
#[cfg(feature = "display")]
mod display;
#[cfg(feature = "display")]
#[doc(inline)]
pub use display::Display;

mod bitmap;
mod rectangle;
#[doc(inline)]
pub use bitmap::*;
#[doc(inline)]
pub use rectangle::*;
