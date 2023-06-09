use std::path::Path;

use async_std::task;
use de_map::{hash::MapHash, io::load_map};

pub fn execute(path: &Path, check: bool) {
    let map = match task::block_on(load_map(path)) {
        Ok(map) => map,
        Err(error) => panic!("Map loading failed: {error:?}"),
    };

    let hash = map.compute_hash();

    if check {
        match MapHash::try_from(path) {
            Ok(path_hash) => {
                if path_hash == hash {
                    println!("Path is valid.");
                } else {
                    panic!("Incorrect file name.");
                }
            }
            Err(error) => panic!("Invalid path: {error:?}"),
        }
    } else {
        println!("{hash:?}");
    }
}
