use bevy::{ecs::system::SystemParam, prelude::*};
use glam::UVec2;

use super::nodes::MinimapNode;

/// This system parameter is capable of construction of [`Drawing`] whose
/// target is the game minimap.
#[derive(SystemParam)]
pub(super) struct DrawingParam<'w, 's> {
    query: Query<'w, 's, &'static UiImage, With<MinimapNode>>,
    images: ResMut<'w, Assets<Image>>,
}

impl<'w, 's> DrawingParam<'w, 's> {
    pub(super) fn drawing(&mut self) -> Drawing {
        let image = self.images.get_mut(&self.query.single().texture).unwrap();
        let size = UVec2::new(
            image.texture_descriptor.size.width,
            image.texture_descriptor.size.height,
        );
        Drawing::new(size, image.data.as_mut_slice())
    }
}

/// This struct holds a mutable reference to RGBA data buffer and implements
/// various drawing methods on it.
pub(super) struct Drawing<'a> {
    size: UVec2,
    data: &'a mut [u8],
}

impl<'a> Drawing<'a> {
    fn new(size: UVec2, data: &'a mut [u8]) -> Self {
        Self { size, data }
    }

    /// Fill whole of the image with a color.
    pub(super) fn fill(&mut self, color: Color) {
        let bytes = color.as_rgba_u32().to_le_bytes();
        for offset in (0..self.data.len()).step_by(4) {
            self.data[offset..(4 + offset)].copy_from_slice(&bytes);
        }
    }

    /// Fill a rectangle with a color.
    pub(super) fn line(&mut self, start: Vec2, end: Vec2, color: Color) {
        panic_bounds("start", start);
        panic_bounds("end", end);

        let start = self.rel_pos_to_px(start);
        let end = self.rel_pos_to_px(end);
        self.line_px(start, end, color);
    }

    /// Fill a rectangle with a color.
    ///
    /// # Panics
    ///
    /// * If `center` is not contained by rectangle (0, 0) -> (1, 1).
    ///
    /// * If `size` has a non-positive coordinate.
    pub(super) fn rect(&mut self, center: Vec2, size: Vec2, color: Color) {
        panic_bounds("center", center);
        if size.cmple(Vec2::ZERO).any() {
            panic!("Both dimensions of size must be positive, got: {size:?}");
        }

        let center = self.rel_pos_to_px(center);

        // Make sure that:
        // * the resulting size in pixels is not depend on `center`
        // * the resulting size in pixels is closest possible to floating point
        //   desired size (i.e. avoid double rounding error)
        let half_size = 0.5 * size * self.size.as_vec2();
        let half_size_rounded = half_size.round();
        let half_size_int = half_size_rounded.as_ivec2();
        let error = half_size - half_size_rounded;
        let correction = (2. * error).round().as_ivec2();
        let top_left = center - half_size_int + correction.min(IVec2::ZERO);
        let bottom_right = center + half_size_int + correction.max(IVec2::ZERO);

        // Make sure that the rectangle is at least 1px large.
        let bottom_right = bottom_right.max(top_left + IVec2::ONE);

        // Make sure that the rectangle is fully within the map.
        let top_left = top_left
            .max(IVec2::ZERO)
            .as_uvec2()
            .min(self.size - UVec2::ONE);
        let bottom_right = bottom_right
            .max(IVec2::ZERO)
            .as_uvec2()
            .min(self.size - UVec2::ONE);

        self.rect_px(top_left, bottom_right, color);
    }

    fn line_px(&mut self, start: IVec2, end: IVec2, color: Color) {
        let bytes = Self::color_to_bytes(color);

        // Bresenham's line algorithm
        let mut x = start.x;
        let mut y = start.y;

        let dx = (end.x - x).abs();
        let dy = -(end.y - y).abs();
        let mut error = dx + dy;
        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };

        loop {
            self.set_pixel_bytes(x as u32, y as u32, bytes);

            if x == end.x && y == end.y {
                break;
            }

            let e2 = 2 * error;
            if e2 >= dy {
                if x == end.x {
                    break;
                }
                error += dy;
                x += sx;
            }
            if e2 <= dx {
                if y == end.y {
                    break;
                }
                error += dx;
                y += sy;
            }
        }
    }

    fn rect_px(&mut self, top_left: UVec2, bottom_right: UVec2, color: Color) {
        let bytes = Self::color_to_bytes(color);
        for y in top_left.y..bottom_right.y {
            for x in top_left.x..bottom_right.x {
                self.set_pixel_bytes(x, y, bytes);
            }
        }
    }

    /// Converts relative coordinates to pixel coordinates.
    fn rel_pos_to_px(&self, point: Vec2) -> IVec2 {
        (point * (self.size.as_ivec2() - IVec2::ONE).as_vec2())
            .round()
            .as_ivec2()
    }

    #[inline]
    fn set_pixel_bytes(&mut self, x: u32, y: u32, bytes: [u8; 4]) {
        let offset = 4 * (y * self.size.x + x) as usize;
        self.data[offset..(4 + offset)].copy_from_slice(&bytes);
    }

    #[inline]
    fn color_to_bytes(color: Color) -> [u8; 4] {
        color.as_rgba_u32().to_le_bytes()
    }
}

fn panic_bounds(name: &str, point: Vec2) {
    if point.cmplt(Vec2::ZERO).any() || point.cmpgt(Vec2::ONE).any() {
        panic!("Coordinates of `{name}` are outside of image bounds.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        let size = UVec2::new(2, 3);
        let mut data = [0u8; 4 * 2 * 3];
        let mut drawing = Drawing::new(size, data.as_mut_slice());
        drawing.fill(Color::rgb(0.5, 0.2, 0.1));

        assert_eq!(
            data,
            [
                127, 51, 25, 255, // (0, 0)
                127, 51, 25, 255, // (1, 0)
                127, 51, 25, 255, // (0, 1)
                127, 51, 25, 255, // (1, 1)
                127, 51, 25, 255, // (0, 2)
                127, 51, 25, 255, // (1, 2)
            ]
        )
    }

    #[test]
    fn test_rect() {
        let size = UVec2::splat(5);
        let mut data = [0u8; 4 * 5 * 5];
        let mut drawing = Drawing::new(size, data.as_mut_slice());
        drawing.rect(
            Vec2::new(0.8, 0.5), // (3.2 -> 3, 2.0 -> 2)
            Vec2::new(0.4, 0.4), // 2x2px
            Color::rgb(0.1, 0.2, 0.1),
        );

        // The rectangle is between (2, 1) and (3, 2).
        assert_eq!(
            data,
            [
                0, 0, 0, 0, // (0, 0)
                0, 0, 0, 0, // (1, 0)
                0, 0, 0, 0, // (2, 0)
                0, 0, 0, 0, // (3, 0)
                0, 0, 0, 0, // (4, 0)
                0, 0, 0, 0, // (0, 1)
                0, 0, 0, 0, // (1, 1)
                25, 51, 25, 255, // (2, 1)
                25, 51, 25, 255, // (3, 1)
                0, 0, 0, 0, // (4, 1)
                0, 0, 0, 0, // (0, 2)
                0, 0, 0, 0, // (1, 2)
                25, 51, 25, 255, // (2, 2)
                25, 51, 25, 255, // (3, 2)
                0, 0, 0, 0, // (4, 2)
                0, 0, 0, 0, // (0, 3)
                0, 0, 0, 0, // (1, 3)
                0, 0, 0, 0, // (2, 3)
                0, 0, 0, 0, // (3, 3)
                0, 0, 0, 0, // (4, 3)
                0, 0, 0, 0, // (0, 4)
                0, 0, 0, 0, // (1, 4)
                0, 0, 0, 0, // (2, 4)
                0, 0, 0, 0, // (3, 4)
                0, 0, 0, 0, // (4, 4)
            ]
        )
    }
}
