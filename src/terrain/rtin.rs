//! Provides implementation of right-triangulated irregular network (RTIN)
//! algorithm. https://www.cs.ubc.ca/~will/papers/rtin.pdf

use crate::terrain::grid::{DiscretePoint, ValueGrid};

/// Partition of a triangle. Triangles are split in two halves along an edge
/// going from midpoint between vertices A and B and vertex C.
///
/// Given a triangle rotated so that vertices A and B lie on axis U and with A
/// on left from point B, left half lies on the left and right side lies on the
/// right.
///
/// In the following example, R is right half and L is left half:
///
///   B
///   |\
///   |R\
///   | /\
///   |/L \
/// C ----- A
#[derive(Clone, Copy, Debug, PartialEq)]
enum Partition {
    Left,
    Right,
}

/// Triangle in a discrete 2D space.
#[derive(Clone, Copy, Debug, PartialEq)]
struct DiscreteTriangle {
    a: DiscretePoint, // left
    b: DiscretePoint, // right
    c: DiscretePoint, // right angle is here
}

impl DiscreteTriangle {
    /// Middle of edge between vertices A and B of the triangle.
    ///
    /// # Panics
    ///
    /// This might panic if the midpoint is not integer valued.
    fn midpoint(&self) -> DiscretePoint {
        (self.a + self.b) / 2
    }

    /// Split the triangle in the middle of edge between vertices A and B and
    /// return one half.
    fn child(&self, partition: Partition) -> DiscreteTriangle {
        match partition {
            Partition::Left => DiscreteTriangle {
                a: self.c,
                b: self.a,
                c: self.midpoint(),
            },
            Partition::Right => DiscreteTriangle {
                a: self.b,
                b: self.c,
                c: self.midpoint(),
            },
        }
    }

    /// Split the triangle in half and return left and right children. See
    /// [DiscreteTriangle::child].
    fn children(&self) -> (DiscreteTriangle, DiscreteTriangle) {
        (self.child(Partition::Left), self.child(Partition::Right))
    }
}

/// Given index of a triangle, return list top square and child triangles
/// partitions necessary to get the triangle.
///
/// The partitioning starts with a top-level square partitioned to bottom-left
/// and upper-right triangles. It follows by splitting triangles on each level
/// to left and right halves. See [Partition].
///
/// Thus `vec![Partition::Left, Partition::Right]` corresponds to right half of
/// the bottom-left triangle.
///
/// Triangle 0 corresponds to the bottom-left triangle. Triangle 1 cor response
/// to the upper-right triangle. Left and right children of any triangle have
/// indices `2 * parent_index + 2` and `2 * parent_index + 3`.
fn get_partitions(mut triangle: u32) -> Vec<Partition> {
    // There is easier binary arithmetic on the index + 2
    triangle += 2;
    // highest level is level 1
    let level = 31 - triangle.leading_zeros();
    // Why reverse: least significant bit in `triangle_id` corresponds to last
    // (finest) triangle division. `partitions` need to start from coarsest.
    // Reversing the integer now is faster than reversing the resulting Vec.
    //
    // Why bit shift: we are not interested in bits beyond `level`. These bits
    // are trailing after the bit reversal.
    triangle = triangle.reverse_bits() >> (32 - level);

    let mut partitions = Vec::with_capacity(level as usize);
    for _i in 0..level {
        if triangle & 1 == 0 {
            partitions.push(Partition::Left)
        } else {
            partitions.push(Partition::Right)
        }
        triangle >>= 1;
    }
    partitions
}

fn max_level_index(level: u8) -> u32 {
    debug_assert!(level <= 31, "Too large level: {}", level);
    // Equivalent to (1 << (level + 1)) - 3 but without overflowing on level 31
    2 * ((1 << level) - 1) - 1
}

/// Builder of a RTIN approximation of an elevation map.
pub struct RtinBuilder<'a> {
    elevation_map: &'a ValueGrid,
    max_error: f32,
    levels: u8,
}

impl<'a> RtinBuilder<'a> {
    /// Create new instance of RTIN builder.
    ///
    /// # Arguments
    ///
    /// * `elevation_map` - elevation map to approximate with a RTIN. Size of
    ///   the elevation map has to be 1 + power of two (e.g. 3, 5 or 33).
    ///
    /// * `max_error` - maximum error, i.e. vertical distance from RTIN to the
    ///   elevation map, allowed for the approximation.
    ///
    /// # Panics
    ///
    /// * If elevation map is smaller than 3.
    ///
    /// * If the given elevation map is not 1 + power of 2.
    pub fn new(elevation_map: &'a ValueGrid, max_error: f32) -> Self {
        // Explicitly state u16 type as the code below relies on that.
        let grid_size: u16 = elevation_map.size() - 1;
        if grid_size < 2 {
            panic!("Grid size has to be at least 2.");
        }
        if (grid_size & (grid_size - 1)) != 0 {
            // This guarantees that triangle vertices on all levels are at
            // integer positions.
            panic!("Grid size has to be a power of 2.");
        }
        // Since `grid_size` is u16, levels cannot be larger than 31, therefore
        // all indices used in this module fit in u32.
        let levels = 1 + (2 * (15 - grid_size.leading_zeros())) as u8;

        Self {
            elevation_map,
            max_error,
            levels,
        }
    }

    /// Calculate RTIN approximation.
    ///
    /// Each four points in an elevation map can be represented by 2 right
    /// triangles (bottom-left and upper-right). This forms grid of 2 *
    /// (elevation_map_size - 1)^2 triangles which is an upper limit of number
    /// of triangles in the resulting RTIN.
    pub fn build(&self) -> Vec<DiscretePoint> {
        let error_map = self.build_error_map();

        let max_non_leaf_index = max_level_index(self.levels - 1);
        let mut stack = vec![0, 1];
        let mut vertices = Vec::new();

        while !stack.is_empty() {
            let triangle_index = stack.pop().unwrap();
            let triangle = self.get_pixel_triangle(triangle_index);

            if triangle_index > max_non_leaf_index
                || error_map.value(triangle.midpoint()) <= self.max_error
            {
                vertices.push(triangle.a);
                vertices.push(triangle.b);
                vertices.push(triangle.c);
            } else {
                stack.push(2 * triangle_index + 2);
                stack.push(2 * triangle_index + 3);
            }
        }
        vertices
    }

    /// Build a value grid which assigns approximation error of each triangle
    /// to midpoint of its hypotenuse.
    fn build_error_map(&self) -> ValueGrid {
        let latest_parent = max_level_index(self.levels - 1);
        let latest_grandparent = max_level_index(self.levels - 2);
        let mut errors = ValueGrid::with_zeros(self.elevation_map.size());

        // Leaf triangles are smallest whole-number triangles and thus have 0
        // error.
        for triangle_index in (0..=latest_parent).rev() {
            let triangle = self.get_pixel_triangle(triangle_index);
            let midpoint = triangle.midpoint();
            let elevation_a = self.elevation_map.value(triangle.a);
            let elevation_b = self.elevation_map.value(triangle.b);
            let elevation_interpolated = (elevation_a + elevation_b) / 2.;
            let elevation_midpoint = self.elevation_map.value(midpoint);
            let mut error = (elevation_interpolated - elevation_midpoint).abs();
            if triangle_index <= latest_grandparent {
                let (left_child, right_child) = triangle.children();
                let left_midpoint = left_child.midpoint();
                let right_midpoint = right_child.midpoint();
                error = error
                    .max(errors.value(midpoint))
                    .max(errors.value(left_midpoint))
                    .max(errors.value(right_midpoint));
            }

            errors.set_value(midpoint, error);
        }
        errors
    }

    /// Calculate triangle (coordinates) from triangle index. See
    /// [get_partitions] for information on triangle partitioning and indexing.
    fn get_pixel_triangle(&self, triangle_index: u32) -> DiscreteTriangle {
        let partitions = get_partitions(triangle_index);

        // There are always at least two top-level triangles.
        let mut triangle = match partitions[0] {
            Partition::Left => DiscreteTriangle {
                // bottom left triangle
                a: DiscretePoint {
                    u: self.elevation_map.size() as u32 - 1,
                    v: self.elevation_map.size() as u32 - 1,
                },
                b: DiscretePoint { u: 0, v: 0 },
                c: DiscretePoint {
                    u: 0,
                    v: self.elevation_map.size() as u32 - 1,
                },
            },
            Partition::Right => DiscreteTriangle {
                // upper right triangle
                a: DiscretePoint { u: 0, v: 0 },
                b: DiscretePoint {
                    u: self.elevation_map.size() as u32 - 1,
                    v: self.elevation_map.size() as u32 - 1,
                },
                c: DiscretePoint {
                    u: self.elevation_map.size() as u32 - 1,
                    v: 0,
                },
            },
        };

        for partition in &partitions[1..] {
            triangle = triangle.child(*partition)
        }
        triangle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_triangle() {
        let bottom_left_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 2, v: 2 },
            b: DiscretePoint { u: 0, v: 0 },
            c: DiscretePoint { u: 0, v: 2 },
        };
        let upper_right_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 0, v: 0 },
            b: DiscretePoint { u: 2, v: 2 },
            c: DiscretePoint { u: 2, v: 0 },
        };
        let left_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 0, v: 0 },
            b: DiscretePoint { u: 0, v: 2 },
            c: DiscretePoint { u: 1, v: 1 },
        };
        let right_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 2, v: 2 },
            b: DiscretePoint { u: 2, v: 0 },
            c: DiscretePoint { u: 1, v: 1 },
        };
        let bottom_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 0, v: 2 },
            b: DiscretePoint { u: 2, v: 2 },
            c: DiscretePoint { u: 1, v: 1 },
        };
        let up_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 2, v: 0 },
            b: DiscretePoint { u: 0, v: 0 },
            c: DiscretePoint { u: 1, v: 1 },
        };
        let left_left_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 1, v: 1 },
            b: DiscretePoint { u: 0, v: 0 },
            c: DiscretePoint { u: 0, v: 1 },
        };
        let left_right_triangle = DiscreteTriangle {
            a: DiscretePoint { u: 0, v: 2 },
            b: DiscretePoint { u: 1, v: 1 },
            c: DiscretePoint { u: 0, v: 1 },
        };
        assert_eq!(
            bottom_left_triangle.children(),
            (bottom_triangle, left_triangle)
        );
        assert_eq!(
            upper_right_triangle.children(),
            (up_triangle, right_triangle)
        );
        assert_eq!(
            left_triangle.children(),
            (left_left_triangle, left_right_triangle)
        );
    }

    #[test]
    fn test_get_partitions() {
        assert_eq!(get_partitions(0), vec![Partition::Left]);
        assert_eq!(get_partitions(1), vec![Partition::Right]);
        assert_eq!(get_partitions(2), vec![Partition::Left, Partition::Left]);
        assert_eq!(get_partitions(3), vec![Partition::Left, Partition::Right]);
        assert_eq!(
            get_partitions(11),
            vec![Partition::Right, Partition::Left, Partition::Right]
        );
    }

    #[test]
    fn test_max_level_index() {
        assert_eq!(max_level_index(1), 1);
        assert_eq!(max_level_index(2), 5);
        assert_eq!(max_level_index(3), 13);
        assert_eq!(max_level_index(4), 29);
    }

    #[test]
    fn test_get_pixel_triangle() {
        let elevation_map = ValueGrid::with_zeros(3);
        let rtin_builder = RtinBuilder::new(&elevation_map, 0.1);
        let triangle = rtin_builder.get_pixel_triangle(11); // right -> left -> right
        let expected = DiscreteTriangle {
            a: DiscretePoint { u: 0, v: 0 },
            b: DiscretePoint { u: 1, v: 1 },
            c: DiscretePoint { u: 1, v: 0 },
        };
        assert_eq!(triangle, expected);
    }

    #[test]
    fn test_build_triangles() {
        let elevation_map = ValueGrid::with_zeros(3);
        let rtin = RtinBuilder::new(&elevation_map, 0.1).build();

        // right direction is first due because the method DFS via stack
        // inverts the ordering
        let expected = vec![
            // upper-right
            DiscretePoint { u: 0, v: 0 },
            DiscretePoint { u: 2, v: 2 },
            DiscretePoint { u: 2, v: 0 },
            // bottom-left
            DiscretePoint { u: 2, v: 2 },
            DiscretePoint { u: 0, v: 0 },
            DiscretePoint { u: 0, v: 2 },
        ];
        assert_eq!(rtin, expected);
    }
}
