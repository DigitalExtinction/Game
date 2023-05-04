use bevy::prelude::*;
use flate2::{Compression, GzBuilder};
use std::fs::File;
use std::path::PathBuf;


pub(crate) struct LogPlugin {
    pub(crate) current_log_file: String,
}

#[derive(Resource)]
pub(crate) struct CurrentLogFile {
    pub(crate) path: String,
}

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLogFile {
            path: self.current_log_file.clone(),
        });
        app.add_startup_system(compress_old_logs);
    }
}

// get all log files in the /logs directory that are not the current log file then compress them and delete the originals
fn compress_old_logs(current_log_file: Res<CurrentLogFile>) {
    let _span = info_span!("Compress_old_logs").entered();

    let current_log_file = current_log_file.path.clone();

    let old_log_files = std::fs::read_dir("logs")
        .unwrap()
        // get the paths
        .map(|res| res.unwrap().path())
        // filter out directories
        .filter(|res| res.is_file())
        // we only want files ending in .log
        .filter(|res| res.file_name().unwrap().to_str().unwrap().ends_with(".log"))
        // we don't want the current log file
        .filter(|res| res.file_name().unwrap().to_str().unwrap() != current_log_file)
        .collect::<Vec<PathBuf>>();
    info!("Compressing old log files: {:?}", old_log_files);

    // compress & delete the old log files

    let time = std::time::Instant::now();
    for old_log_file_path in old_log_files {
        let compressed_log_file =
            File::create(old_log_file_path.to_str().unwrap().to_string() + ".gz").unwrap();
        let mut old_log_file_handle = File::open(old_log_file_path.clone()).unwrap();
        let mut gz = GzBuilder::new()
            .filename("log.txt")
            .comment("test file, please delete")
            .write(compressed_log_file, Compression::best());
        debug!("Compressing old log file: {:?}", old_log_file_path);
        std::io::copy(&mut old_log_file_handle, &mut gz).unwrap();
        gz.finish().unwrap();

        debug!("Deleting old log file: {:?}", old_log_file_path);
        std::fs::remove_file(old_log_file_path).unwrap();
    }
    info!(
        "Compressed old log files in {}ms",
        time.elapsed().as_millis()
    );
}

// --------- tests ---------
#[cfg(test)]
mod log_tests {
    use std::io::Write;
    use super::*;
    use bevy::utils::tracing;

    // does the compression system work?
    #[test]
    fn test_compress_old_logs() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();
        let mut app = App::new();
        app.add_plugin(LogPlugin {
            current_log_file: "test_current.log".to_string(),
        });
        // make sure logs directory exists
        std::fs::create_dir_all("logs").unwrap();
        // create 2 uncompressed log files and 1 compressed log file in the logs directory (check that the uncompressed files change and get .gz added to the end but the compressed ones dont change)
        let mut uncompressed_log_file_1 =
            File::create("logs/test_uncompressed_log_file_1.log").unwrap();
        trace!("file 1 created: {:?}", uncompressed_log_file_1);

        let mut uncompressed_log_file_2 =
            File::create("logs/test_uncompressed_log_file_2.log").unwrap();
        trace!("file 2 created: {:?}", uncompressed_log_file_2);

        let mut compressed_log_file = File::create("logs/test_compressed_log_file.log.gz").unwrap();
        trace!("file 3 created: {:?}", compressed_log_file);

        // --- write test data ---
        let mut current_log = File::create("logs/test_current.log").unwrap();
        uncompressed_log_file_1.write_all(b"test").unwrap();

        let large_test_bytes = b"test".repeat(300000);
        uncompressed_log_file_2
            .write_all(&*large_test_bytes)
            .unwrap();

        compressed_log_file.write_all(b"test").unwrap();
        current_log.write_all(b"test").unwrap();

        // --- run app ---
        app.run();
        println!("aba");

        // --- check results ---
        // only 4 files should be in the logs directory (old ones deleted, new compressed ones added) and the current log
        assert_eq!(
            std::fs::read_dir("logs")
                .expect("Could not read logs directory. Does it exist?")
                // we only want files with test_ at the start
                .filter_map(|file| {
                    if file
                        .expect("Could not read DirEntry")
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with("test_")
                    {
                        Some(())
                    } else {
                        None
                    }
                })
                .count(),
            4
        );

        // have the uncompressed log files been compressed?
        assert_eq!(
            std::fs::read_dir("logs")
                .expect("Could not read logs directory. Does it exist?")
                // we only want files with test_ at the start
                .filter_map(|file| {
                    if file
                        .as_ref()
                        .expect("Could not read DirEntry")
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with("test_")
                    {
                        Some(file.unwrap())
                    } else {
                        None
                    }
                })
                // we only want files ending in .gz
                .filter_map(|file| {
                    if file.file_name().to_str().unwrap().ends_with(".log") {
                        Some(())
                    } else {
                        None
                    }
                })
                .count(),
            // they should all be gz now except for current
            1
        );
        // does the content now differ from the original?
        assert_ne!(
            std::fs::read("logs/test_uncompressed_log_file_1.log.gz")
                .expect("Could not read compressed file. Was it created?"),
            b"test"
        );
        assert_ne!(
            std::fs::read("logs/test_uncompressed_log_file_2.log.gz")
                .expect("Could not read compressed file. Was it created?"),
            large_test_bytes
        );
        // does the compressed log file still exist and have same content?
        assert_eq!(
            std::fs::read("logs/test_compressed_log_file.log.gz")
                .expect("Could not read compressed file. Was it created?"),
            b"test"
        );
        // does the current log file still exist and have same content?
        assert_eq!(
            std::fs::read("logs/test_current.log")
                .expect("Could not read compressed file. Was it created?"),
            b"test"
        );

        // --- cleanup ---
        for file in std::fs::read_dir("logs")
            .expect("Could not read logs directory. Does it exist?")
            // we only want files with test_ at the start
            .filter_map(|file| {
                if file
                    .as_ref()
                    .expect("Could not read DirEntry")
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with("test_")
                {
                    Some(file)
                } else {
                    None
                }
            })
        {
            std::fs::remove_file(file.unwrap().path()).expect("Could not delete file");
        }
    }
}
