use std::{cmp::Ordering, ops::Range};

use bevy::{
    asset::Asset,
    prelude::{Handle, Image, Material},
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};
use glam::{Mat3, Vec2};

// * Keep this in sync with terrain.wgsl.
// * Keep this smaller or equal to de_types::objects::PLAYER_MAX_UNITS.
pub(crate) const CIRCLE_CAPACITY: usize = 127;
// * Keep this in sync with terrain.wgsl.
// * Keep this smaller or equal to de_types::objects::PLAYER_MAX_BUILDINGS.
pub(crate) const RECTANGLE_CAPACITY: usize = 31;

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "9e124e04-fdf1-4836-b82d-fa2f01fddb62"]
pub struct TerrainMaterial {
    #[uniform(0)]
    circles: KdTree,
    #[uniform(1)]
    rectangles: Rectangles,
    #[texture(2)]
    #[sampler(3)]
    texture: Handle<Image>,
}

impl TerrainMaterial {
    pub(crate) fn new(texture: Handle<Image>) -> Self {
        Self {
            circles: KdTree::empty(),
            rectangles: Rectangles::default(),
            texture,
        }
    }

    pub(crate) fn set_circle_markers(&mut self, circles: Vec<Circle>) {
        self.circles.rebuild(circles);
    }

    pub(crate) fn set_rectangle_markers(&mut self, rectangles: Vec<Rectangle>) {
        self.rectangles.set_rectangles(rectangles);
    }
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}

#[derive(ShaderType, Debug, Clone, Copy, Default)]
pub(crate) struct Circle {
    #[align(16)]
    center: Vec2,
    radius: f32,
}

impl Circle {
    /// Creates a new circle.
    ///
    /// # Panics
    ///
    /// * If `center` is not finite.
    /// * If radius is non finite or is smaller or equal to zero.
    pub(crate) fn new(center: Vec2, radius: f32) -> Self {
        if !center.is_finite() {
            panic!("Circle center is not finite: {center:?}");
        }
        if !radius.is_finite() {
            panic!("Circle radius is not finite: {radius:?}");
        }
        if radius <= 0. {
            panic!("Circle radius is smaller or equal to 0: {radius:?}");
        }

        Self { center, radius }
    }

    fn coord(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.center.x,
            Axis::Y => self.center.y,
        }
    }
}

#[derive(Copy, Clone)]
enum Axis {
    X,
    Y,
}

impl Axis {
    fn toggle(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::X,
        }
    }
}

#[derive(ShaderType, Debug, Clone)]
struct KdTree {
    /// Nodes of the KD tree. Given a node at `index` its left child is at `2 *
    /// index + 1` and right child is at `2 * index + 2`.
    #[align(16)]
    nodes: [Circle; CIRCLE_CAPACITY],
    count: u32,
}

impl KdTree {
    fn empty() -> Self {
        Self {
            nodes: [Circle::default(); CIRCLE_CAPACITY],
            count: 0,
        }
    }

    /// Rebuids the KD tree from a vector of circles.
    ///
    /// # Panics
    ///
    /// Panics if the number of circles is larger than maximum allowed
    /// capacity.
    fn rebuild(&mut self, mut circles: Vec<Circle>) {
        if circles.len() > CIRCLE_CAPACITY {
            panic!(
                "Number of circles {} is greater than shader capacity {}.",
                circles.len(),
                CIRCLE_CAPACITY
            );
        }

        self.count = circles.len() as u32;
        if self.count == 0 {
            return;
        }

        struct StackItem {
            index: u16,
            axis: Axis,
            range: Range<usize>,
        }

        let mut stack: Vec<StackItem> = vec![StackItem {
            index: 0,
            axis: Axis::X,
            range: 0..circles.len(),
        }];

        while let Some(item) = stack.pop() {
            let subtree = &mut circles[item.range.clone()];

            subtree.sort_by(|a, b| {
                let a = a.coord(item.axis);
                let b = b.coord(item.axis);

                if a < b {
                    Ordering::Less
                } else if a > b {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });

            // A median cannot be used because it is desired that that the
            // three occupies continuous range in tree.nodes (from 0 to
            // circles.len()).
            //
            // The following computation guarantees that that the tree is
            // balanced (the max diff between tree depth is 1) and that it each
            // level/floor is filled from "left to right".
            let num_bits = 31 - (subtree.len() as u32).leading_zeros();
            let max_subtree_size = (1 << num_bits) - 1;
            let min_subtree_size = max_subtree_size >> 1;
            let split_index = (subtree.len() - min_subtree_size - 1).min(max_subtree_size);
            self.nodes[item.index as usize] = subtree[split_index];

            if split_index < (subtree.len() - 1) {
                stack.push(StackItem {
                    index: 2 * item.index + 2,
                    axis: item.axis.toggle(),
                    range: (item.range.start + split_index + 1)..item.range.end,
                });
            }
            if split_index > 0 {
                stack.push(StackItem {
                    index: 2 * item.index + 1,
                    axis: item.axis.toggle(),
                    range: item.range.start..(item.range.start + split_index),
                });
            }
        }
    }
}

#[derive(ShaderType, Debug, Clone, Copy, Default)]
pub(crate) struct Rectangle {
    pub(crate) inverse_transform: Mat3,
    pub(crate) half_size: Vec2,
}

impl Rectangle {
    pub(crate) fn new(inverse_transform: Mat3, half_size: Vec2) -> Self {
        Self {
            inverse_transform,
            half_size,
        }
    }
}

#[derive(ShaderType, Debug, Clone)]
struct Rectangles {
    items: [Rectangle; RECTANGLE_CAPACITY],
    count: u32,
}

impl Rectangles {
    /// Sets the rectangles held in the list.
    ///
    /// # Panics
    ///
    /// Panics if the number of rectangles is larger than maximum allowed
    /// capacity.
    fn set_rectangles(&mut self, rectangles: Vec<Rectangle>) {
        if rectangles.len() > RECTANGLE_CAPACITY {
            panic!(
                "Number of rectangles {} is greater than shader capacity {}.",
                rectangles.len(),
                RECTANGLE_CAPACITY
            );
        }

        self.items[..rectangles.len()].copy_from_slice(&rectangles);
        self.count = rectangles.len() as u32;
    }
}

impl Default for Rectangles {
    fn default() -> Self {
        Self {
            items: [Rectangle::default(); RECTANGLE_CAPACITY],
            count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_kd_tree_build_many() {
        let mut circles = vec![
            Circle::new(Vec2::new(1., -4.), 1.),
            Circle::new(Vec2::new(-2., 1.), 1.),
            Circle::new(Vec2::new(-1.5, -3.), 1.),
            Circle::new(Vec2::new(2., 1.), 1.),
        ];

        let mut tree = KdTree::empty();

        for permutation in (0..circles.len()).permutations(circles.len()) {
            let version: Vec<Circle> = permutation.iter().map(|index| circles[*index]).collect();
            tree.rebuild(version);

            assert_eq!(tree.count, 4);
            assert_eq!(tree.nodes[0].center, Vec2::new(1., -4.));
            assert_eq!(tree.nodes[1].center, Vec2::new(-2., 1.));
            assert_eq!(tree.nodes[2].center, Vec2::new(2., 1.));
            assert_eq!(tree.nodes[3].center, Vec2::new(-1.5, -3.));
        }

        circles.push(Circle::new(Vec2::new(-8., 2.), 1.));
        for permutation in (0..circles.len()).permutations(circles.len()) {
            let version: Vec<Circle> = permutation.iter().map(|index| circles[*index]).collect();
            tree.rebuild(version);

            assert_eq!(tree.count, 5);
            assert_eq!(tree.nodes[0].center, Vec2::new(1., -4.));
            assert_eq!(tree.nodes[1].center, Vec2::new(-2., 1.));
            assert_eq!(tree.nodes[2].center, Vec2::new(2., 1.));
            assert_eq!(tree.nodes[3].center, Vec2::new(-1.5, -3.));
            assert_eq!(tree.nodes[4].center, Vec2::new(-8., 2.));
        }

        circles.push(Circle::new(Vec2::new(1.5, 0.), 1.));
        for permutation in (0..circles.len()).permutations(circles.len()) {
            let version: Vec<Circle> = permutation.iter().map(|index| circles[*index]).collect();
            tree.rebuild(version);

            assert_eq!(tree.count, 6);
            assert_eq!(tree.nodes[0].center, Vec2::new(1., -4.));
            assert_eq!(tree.nodes[1].center, Vec2::new(-2., 1.));
            assert_eq!(tree.nodes[2].center, Vec2::new(2., 1.));
            assert_eq!(tree.nodes[3].center, Vec2::new(-1.5, -3.));
            assert_eq!(tree.nodes[4].center, Vec2::new(-8., 2.));
            assert_eq!(tree.nodes[5].center, Vec2::new(1.5, 0.));
        }
    }
}
