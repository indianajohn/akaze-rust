extern crate image;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate primal;

use image::GenericImage;
use image::Luma;
use image::Pixel;
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

/// This function computes the Perona and Malik conductivity coefficient g2
/// g2 = 1 / (1 + dL^2 / k^2)
/// `Lx` First order image derivative in X-direction (horizontal)
/// `Ly` First order image derivative in Y-direction (vertical)
/// `k` Contrast factor parameter
/// # Return value
/// Output image
#[allow(non_snake_case)]
fn pm_g2(Lx: &GrayFloatImage, Ly: &GrayFloatImage, k: f64) -> GrayFloatImage {
    let mut dst = GrayFloatImage::new(Lx.width(), Lx.height());
    assert!(Lx.width() == Ly.width());
    assert!(Lx.height() == Ly.height());
    let inverse_k: f64 = 1.0f64 / (k * k);
    for y in 0..Lx.height() {
        for x in 0..Lx.width() {
            let Lx_pixel: f64 = Lx.get_pixel(x, y).channels()[0] as f64;
            let Ly_pixel: f64 = Ly.get_pixel(x, y).channels()[0] as f64;
            let dst_pixel: f64 =
                1.0f64 / (1.0f64 + inverse_k * (Lx_pixel * Lx_pixel + Ly_pixel * Ly_pixel));
            dst.put_pixel(x, y, Luma([dst_pixel as f32]));
        }
    }
    dst
}

fn create_nonlinear_scale_space(
    evolutions: &mut Vec<EvolutionStep>,
    image: &GrayFloatImage,
    options: Config,
) {
    info!("Creating first evolution.");
    evolutions[0].Lt = image::imageops::blur(image, options.base_scale_offset as f32);
    evolutions[0].Lsmooth = evolutions[0].Lt.clone();
    let contrast_factor = ops::contrast_factor::compute_contrast_factor(
        &evolutions[0].Lsmooth,
        options.contrast_percentile,
        1.0f64,
        options.contrast_factor_num_bins,
    );
    for i in 1..evolutions.len() {
        info!("Creating evolution {}.", i);
        if evolutions[i].octave > evolutions[i - 1].octave {
            evolutions[i].Lt = image::imageops::resize(
                &evolutions[i - 1].Lt,
                evolutions[i].Lt.width(),
                evolutions[i].Lt.height(),
                image::FilterType::Gaussian,
            );
        } else {
            evolutions[i].Lt = evolutions[i - 1].Lt.clone();
        }
        evolutions[0].Lsmooth = image::imageops::blur(&evolutions[0].Lt, 1.0f32);
        evolutions[i].Lx = ops::derivatives::scharr(&evolutions[0].Lsmooth, true, false);
        evolutions[i].Ly = ops::derivatives::scharr(&evolutions[0].Lsmooth, false, true);
        evolutions[i].Lflow = pm_g2(&evolutions[i].Lx, &evolutions[i].Ly, contrast_factor);
        for j in 0..evolutions[i].fed_tau_steps.len() {
            let step_size: f64 = evolutions[i].fed_tau_steps[j];
            ops::nonlinear_diffusion::calculate_step(&mut evolutions[i], step_size);
        }
    }
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
