extern crate akaze;
use std::path::Path;
use std::path::PathBuf;
#[macro_use]
extern crate log;
extern crate env_logger;
use std::time::SystemTime;

use akaze::types::evolution::{Config, write_evolutions};

fn locate_test_data() -> PathBuf {
    let exe_path = ::std::env::current_exe().unwrap();
    let mut parent_path = exe_path.parent().unwrap().to_owned();
    parent_path.push("../../../test-data/akaze-test-data");
    parent_path
}

#[test]
fn test_locate_data() {
    warn!(
        "Note: test data can be obtained from the akaze-test-data
        repository See README.md"
    );
    let test_data_path = locate_test_data();
    let mut image_file_path = test_data_path;
    image_file_path.push("1.jpg");
    let metadata = ::std::fs::metadata(image_file_path).unwrap();
    debug_assert!(metadata.is_file());
    let test_data_path = locate_test_data();
    let mut image_file_path = test_data_path;
    image_file_path.push("2.jpg");
    let metadata = ::std::fs::metadata(image_file_path).unwrap();
    debug_assert!(metadata.is_file());
    let test_data_path = locate_test_data();
    let mut image_file_path = test_data_path;
    image_file_path.push("1-output");
    let metadata = ::std::fs::metadata(image_file_path).unwrap();
    debug_assert!(metadata.is_dir());
}

#[test]
fn extract_features() {
    let start = SystemTime::now();
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");
    env_logger::Builder::from_env(env).init();
    let mut test_image_path = locate_test_data();
    test_image_path.push("1.jpg");
    // TODO: temp dir
    let output_path = Path::new("output.json");
    let options = Config::default();
    let evolutions = akaze::extract_features(test_image_path, output_path.to_owned(), options);
    match std::env::var("AKAZE_SCALE_SPACE_DIR") {
        Ok(val) => {
            info!("Writing scale space; if you want to skip this step, undefine the env var AKAZE_SCALE_SPACE_DIR");
            let string_to_pass = val.to_string();
            std::fs::create_dir_all(&string_to_pass.clone()).unwrap();
            write_evolutions(
                &evolutions,
                std::path::Path::new(&string_to_pass.clone()).to_owned(),
            );
        }
        Err(_e) => {
            info!("Not writing scale space; do write scale space, define the env var AKAZE_SCALE_SPACE_DIR");
        }
    }
    match std::fs::remove_file(output_path) {
        Err(result) => warn!("Could not clean up temp files; returned error {:?}", result),
        Ok(result) => trace!("Cleaned up temp files with result: {:?}", result),
    };
    info!("Total duration: {:?}", start.elapsed().unwrap());
}
