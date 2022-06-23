use std::path::PathBuf;

use clap::Parser;
use parry3d::{bounding_volume::AABB, math::Point};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser, help = "Path of a GLTF file.")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let (document, buffers, _images) = match gltf::import(args.path.as_path()) {
        Ok(loaded) => loaded,
        Err(err) => panic!("GLTF loading error: {:?}", err),
    };
    let get_buffer_data = |buffer: gltf::Buffer| buffers.get(buffer.index()).map(|x| &*x.0);

    let (min, max) = document
        .meshes()
        .flat_map(|mesh| mesh.primitives())
        .flat_map(|primitive| primitive.reader(get_buffer_data).read_positions().unwrap())
        .fold(
            (
                [f32::INFINITY, f32::INFINITY, f32::INFINITY],
                [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY],
            ),
            |mut acc, item| {
                for (i, &coord) in item.iter().enumerate() {
                    acc.0[i] = acc.0[i].min(coord);
                    acc.1[i] = acc.1[i].max(coord);
                }
                acc
            },
        );

    let (positions, indices) = AABB::new(Point::from(min), Point::from(max)).to_trimesh();
    println!("Positions: {:?}", positions);
    println!("Indices: {:?}", indices);
}
