/// A rectangled, defined by top-left (inclusive) and bottom-right (exclusive)
/// coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rectangle {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rectangle {
    /// Expands the rectangle by the given number of pixels in all four
    /// directions and returns the resulting rectangle. (The width and height
    /// will increase by twice the given amount.)
    pub fn expand_by(&self, amount: u32) -> Rectangle {
        Rectangle {
            left: self.left - amount as i32,
            top: self.top - amount as i32,
            right: self.right + amount as i32,
            bottom: self.bottom + amount as i32,
        }
    }
    /// Returns the rectangle that exists at the intersection between these two
    /// rectangles. The result will be an empty rectangle if there is no
    /// overlap.
    pub fn intersection(&self, rhs: Rectangle) -> Rectangle {
        if self.is_empty() || rhs.is_empty() {
            Rectangle::default()
        } else {
            Rectangle {
                left: self.left.max(rhs.left),
                top: self.top.max(rhs.top),
                right: self.right.min(rhs.right),
                bottom: self.bottom.min(rhs.bottom),
            }
        }
    }
    /// Returns the smallest rectangle that contains both of the source
    /// rectangles. (It will likely also contain some pixels that are outside
    /// of either rectangle.)
    pub fn union(&self, rhs: Rectangle) -> Rectangle {
        if self.is_empty() {
            rhs
        } else if rhs.is_empty() {
            *self
        } else {
            Rectangle {
                left: self.left.min(rhs.left),
                top: self.top.min(rhs.top),
                right: self.right.max(rhs.right),
                bottom: self.bottom.max(rhs.bottom),
            }
        }
    }
    /// Returns true if this rectangle doesn't contain anything (width or
    /// height will be zero).
    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }
    /// Returns the number of pixels covered horizontally by this rectangle.
    pub fn get_width(&self) -> u32 {
        if self.right <= self.left {
            0
        } else {
            (self.right - self.left) as u32
        }
    }
    /// Returns the number of pixels covered vertically by this rectangle.
    pub fn get_height(&self) -> u32 {
        if self.bottom <= self.top {
            0
        } else {
            (self.bottom - self.top) as u32
        }
    }
    /// Returns the total number of pixels covered by this rectangle.
    pub fn get_area(&self) -> u32 {
        self.get_width() * self.get_height()
    }
}
