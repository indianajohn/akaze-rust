
extern crate akaze;
use std::path::Path;
use std::path::PathBuf;

fn locate_test_data() -> PathBuf {
    let exe_path = ::std::env::current_exe().unwrap();
    let mut parent_path = exe_path.parent().unwrap().to_owned();
    parent_path.push("../../../test-data/akaze-test-data");
    parent_path
}

#[test]
fn extract_features() {
    let mut test_image_path = locate_test_data();
    test_image_path.push("1.jpg");
    // TODO: temp dir
    let output_path = Path::new("output.png");
    let options = akaze::Config::default();
    akaze::extract_features(test_image_path, output_path.to_owned(), options);
}