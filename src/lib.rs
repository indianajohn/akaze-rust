extern crate image;
#[macro_use]
extern crate log;
extern crate env_logger;

use image::GenericImage;
use std::path::PathBuf;

pub mod ops;
pub mod types;
use types::evolution::Config;
use types::evolution::EvolutionStep;
use types::image::GrayFloatImage;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
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

fn create_nonlinear_scale_space(
    evolutions: &mut Vec<EvolutionStep>,
    image: &GrayFloatImage,
    options: Config,
) {
    evolutions[0].Lt = image::imageops::blur(image, options.base_scale_offset as f32);
    evolutions[0].Lsmooth = evolutions[0].Lt.clone();
    let mut _contrast_factor = ops::contrast_factor::compute_contrast_factor(
        &evolutions[0].Lsmooth,
        options.contrast_percentile,
        1.0f64,
        options.contrast_factor_num_bins,
    );
    warn!("TODO: finish");
}

/// Extract features using the Akaze feature extractor.
///
/// # Arguments
/// `input_image_path` - the input image for which to extract features.
/// `output_features_path` - the output path to which to write an output JSON file.
/// `options: the options for the algorithm.`
///
pub fn extract_features(input_image_path: PathBuf, output_features_path: PathBuf, options: Config) {
    let input_image = image::open(input_image_path).unwrap();
    let float_image = types::image::create_unit_float_image(&input_image);
    info!(
        "Loaded a {} x {} image",
        input_image.width(),
        input_image.height()
    );
    let mut evolutions =
        types::evolution::allocate_evolutions(input_image.width(), input_image.height(), options);
    create_nonlinear_scale_space(&mut evolutions, &float_image, options);
    warn!("TODO: finish");
    std::fs::write(output_features_path, "foo").unwrap(); // placeholder
}
