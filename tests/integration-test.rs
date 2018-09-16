extern crate akaze;
use std::path::Path;
use std::path::PathBuf;
#[macro_use]
extern crate log;
extern crate env_logger;

use akaze::types::evolution::Config;

fn locate_test_data() -> PathBuf {
    let exe_path = ::std::env::current_exe().unwrap();
    let mut parent_path = exe_path.parent().unwrap().to_owned();
    parent_path.push("../../../test-data/akaze-test-data");
    parent_path
}

#[test]
fn extract_features() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");
    env_logger::Builder::from_env(env).init();
    let mut test_image_path = locate_test_data();
    test_image_path.push("1.jpg");
    // TODO: temp dir
    let output_path = Path::new("output.json");
    let options = Config::default();
    akaze::extract_features(test_image_path, output_path.to_owned(), options);
    match std::fs::remove_file(output_path) {
        Err(result) => warn!("Could not clean up temp files; returned error {:?}", result),
        Ok(result) => trace!("Cleaned up temp files with result: {:?}", result),
    };
}
