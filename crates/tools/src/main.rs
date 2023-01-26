use std::path::PathBuf;

use clap::Parser;
use glam::{Mat4, Vec3};
use gltf::Node;
use parry3d::{bounding_volume::Aabb, math::Point};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser, help = "Path of a GLTF file.")]
    path: PathBuf,
}

struct WorldNode<'a> {
    node: Node<'a>,
    transform: Mat4,
}

impl<'a> WorldNode<'a> {
    fn from_node(node: Node<'a>) -> Self {
        let transform = Mat4::from_cols_array_2d(&node.transform().matrix());
        Self { node, transform }
    }

    fn node(&self) -> &Node<'a> {
        &self.node
    }

    fn new_child(&self, child: Node<'a>) -> Self {
        let child_transform = Mat4::from_cols_array_2d(&child.transform().matrix());
        Self {
            node: child,
            transform: self.transform * child_transform,
        }
    }
}

fn main() {
    let args = Args::parse();

    let (document, buffers, _images) = match gltf::import(args.path.as_path()) {
        Ok(loaded) => loaded,
        Err(err) => panic!("GLTF loading error: {err:?}"),
    };
    let get_buffer_data = |buffer: gltf::Buffer| buffers.get(buffer.index()).map(|x| &*x.0);

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for scene in document.scenes() {
        let mut stack = Vec::new();
        stack.extend(scene.nodes().map(WorldNode::from_node));

        while !stack.is_empty() {
            let world_node = stack.pop().unwrap();
            let node = world_node.node();

            stack.extend(node.children().map(|c| world_node.new_child(c)));

            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    for position in primitive.reader(get_buffer_data).read_positions().unwrap() {
                        let position = Vec3::from_array(position);
                        min = min.min(position);
                        max = max.max(position);
                    }
                }
            }
        }
    }

    let (positions, indices) = Aabb::new(
        Point::new(min.x, min.y, min.z),
        Point::new(max.x, max.y, max.z),
    )
    .to_trimesh();
    println!("Positions: {positions:?}");
    println!("Indices: {indices:?}");
}
