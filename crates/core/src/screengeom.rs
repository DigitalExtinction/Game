use glam::Vec2;

/// A rectangle on the screen.
#[derive(Copy, Clone)]
pub struct ScreenRect(Vec2, Vec2);

impl ScreenRect {
    pub fn full() -> Self {
        Self(-Vec2::ONE, Vec2::ONE)
    }

    /// Creates a new screen rectangle from two arbitrary points which lie
    /// between [-1, -1] and [1, 1].
    ///
    /// See [`Self::new`].
    pub fn from_points(a: Vec2, b: Vec2) -> Self {
        Self::new(a.min(b), a.max(b))
    }

    /// # Arguments
    ///
    /// * `bottom_left` - coordinates of the bottom left corner of the rectangle.
    ///   Bottom-left corner has coordinates [-1, -1] and top-right corner has
    ///   coordinates [1, 1].
    ///
    /// * `top_right` - see `bottom_left`
    ///
    /// # Panics
    ///
    /// * If one of the corners is outside of screen space boundaries.
    ///
    /// * If `bottom_left` corner is to the right or to the top from
    ///   `top_right` corner.
    pub fn new(bottom_left: Vec2, top_right: Vec2) -> Self {
        if bottom_left.cmpgt(top_right).any() {
            panic!(
                "Bottom left corner is greater than top right corner: {bottom_left:?} > {top_right:?}"
            );
        }

        if bottom_left.abs().cmpgt(Vec2::ONE).any() {
            panic!("Bottom left corner is not within screen range: {bottom_left:?}");
        }
        if top_right.abs().cmpgt(Vec2::ONE).any() {
            panic!("Top right corner is not within screen range: {top_right:?}");
        }
        Self(bottom_left, top_right)
    }

    /// Returns array of edge coordinates corresponding to (in order):
    ///
    /// * X coordinate of the left edge.
    /// * X coordinate of the right edge.
    /// * Y coordinate of the bottom edge.
    /// * Y coordinate of the top edge.
    pub fn as_array(&self) -> [f32; 4] {
        [self.left(), self.right(), self.bottom(), self.top()]
    }

    /// X coordinate of the left edge.
    pub fn left(&self) -> f32 {
        self.0.x
    }

    /// X coordinate of the right edge.
    pub fn right(&self) -> f32 {
        self.1.x
    }

    /// Y coordinate of the bottom edge.
    pub fn bottom(&self) -> f32 {
        self.0.y
    }

    /// Y coordinate of the top edge.
    pub fn top(&self) -> f32 {
        self.1.y
    }

    /// Returns size of the rectangle. Full screen size is [2., 2.].
    pub fn size(&self) -> Vec2 {
        self.1 - self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_points() {
        let rect = ScreenRect::from_points(Vec2::new(0.1, 0.2), Vec2::new(-0.15, 0.3));
        assert_eq!(rect.left(), -0.15);
        assert_eq!(rect.right(), 0.1);
        assert_eq!(rect.bottom(), 0.2);
        assert_eq!(rect.top(), 0.3);
    }
}
