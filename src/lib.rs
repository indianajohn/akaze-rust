
extern crate image;
extern crate imageproc;

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    fn locate_test_data() -> PathBuf {
        let exe_path = ::std::env::current_exe().unwrap();
        let mut parent_path = exe_path.parent().unwrap().to_owned();
        parent_path.push("../../../test-data");
        parent_path
    }

    #[test]
    fn test_locate_data() {
        println!(
            "Note: test data can be obtained from the akaze-test-data
            repository See README.md");
        let test_data_path = locate_test_data();
        let mut image_file_path = test_data_path;
        image_file_path.push("1.jpg");
        let metadata = ::std::fs::metadata(image_file_path).unwrap();
        assert!(metadata.is_file());
        let test_data_path = locate_test_data();
        let mut image_file_path = test_data_path;
        image_file_path.push("2.jpg");
        let metadata = ::std::fs::metadata(image_file_path).unwrap();
        assert!(metadata.is_file());
        let test_data_path = locate_test_data();
        let mut image_file_path = test_data_path;
        image_file_path.push("1-output");
        let metadata = ::std::fs::metadata(image_file_path).unwrap();
        assert!(metadata.is_dir());
    }
}

/// Extract features using the Akaze feature extractor.
/// 
/// # Arguments
/// `_input_image_path` - the input image for which to extract features.
/// `_output_features_path` - the output path to which to write an output JSON file.
/// 
pub fn extract_features(_input_image_path: PathBuf, _output_features_path: PathBuf) {
    // TODO
}
