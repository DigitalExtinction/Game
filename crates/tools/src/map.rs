use std::path::Path;

use async_std::task;
use de_map::io::load_map;

pub fn execute(path: &Path) {
    let map = match task::block_on(load_map(path)) {
        Ok(map) => map,
        Err(error) => panic!("Map loading failed: {error:?}"),
    };

    let hash = map.compute_hash();
    println!("{hash:?}");
}
