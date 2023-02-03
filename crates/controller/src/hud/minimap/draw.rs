use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};

use super::MapImageHandle;

/// This system parameter is capable of construction of [`Drawing`] whose
/// target is the game minimap.
#[derive(SystemParam)]
pub(super) struct DrawingParam<'w, 's> {
    handle: Res<'w, MapImageHandle>,
    images: ResMut<'w, Assets<Image>>,
    #[system_param(ignore)]
    marker: PhantomData<&'s ()>,
}

impl<'w, 's> DrawingParam<'w, 's> {
    pub(super) fn drawing(&mut self) -> Drawing {
        let image = self.images.get_mut(&self.handle.0).unwrap();
        Drawing::new(image.data.as_mut_slice())
    }
}

/// This struct holds a mutable reference to RGBA data buffer and implements
/// various drawing methods on it.
pub(super) struct Drawing<'a> {
    data: &'a mut [u8],
}

impl<'a> Drawing<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        Self { data }
    }

    /// Fill whole of the image with a color.
    pub(super) fn fill(&mut self, color: Color) {
        let bytes = color.as_rgba_u32().to_le_bytes();
        for offset in (0..self.data.len()).step_by(4) {
            self.data[offset..(4 + offset)].copy_from_slice(&bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        // width 2, height 3
        let mut data = [0u8; 4 * 2 * 3];
        let mut drawing = Drawing::new(data.as_mut_slice());
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
}
