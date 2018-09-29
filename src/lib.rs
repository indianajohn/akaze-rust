extern crate image;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate primal;
extern crate time;
extern crate num_cpus;
extern crate scoped_threadpool;

use image::GenericImageView;
use std::path::PathBuf;
use time::PreciseTime;

pub mod ops;
pub mod types;
use types::evolution::Config;
use types::evolution::EvolutionStep;
use types::image::gaussian_blur;
use types::image::{GrayFloatImage, ImageFunctions};

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
    debug_assert!(Lx.width() == Ly.width());
    debug_assert!(Lx.height() == Ly.height());
    let inverse_k: f64 = 1.0f64 / (k * k);
    for y in 0..Lx.height() {
        for x in 0..Lx.width() {
            let Lx_pixel: f64 = Lx.get(x, y) as f64;
            let Ly_pixel: f64 = Ly.get(x, y) as f64;
            let dst_pixel: f64 =
                1.0f64 / (1.0f64 + inverse_k * (Lx_pixel * Lx_pixel + Ly_pixel * Ly_pixel));
            dst.put(x, y, dst_pixel as f32);
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
    let start = PreciseTime::now();
    evolutions[0].Lt = gaussian_blur(image, options.base_scale_offset as f32, 5);
    debug!("Gaussian blur took {}.", start.to(PreciseTime::now()));
    evolutions[0].Lsmooth = evolutions[0].Lt.clone();
    debug!(
        "Convolving first evolution with sigma={} Gaussian.",
        options.base_scale_offset
    );
    let start = PreciseTime::now();
    let mut contrast_factor = ops::contrast_factor::compute_contrast_factor(
        &evolutions[0].Lsmooth,
        options.contrast_percentile,
        1.0f64,
        options.contrast_factor_num_bins,
    );
    debug!(
        "Computing contrast factor took {}.",
        start.to(PreciseTime::now())
    );
    debug!(
        "Contrast percentile={}, Num bins={}, Initial contrast factor={}",
        options.contrast_percentile, options.contrast_factor_num_bins, contrast_factor
    );
    for i in 1..evolutions.len() {
        info!("Creating evolution {}.", i);
        if evolutions[i].octave > evolutions[i - 1].octave {
            evolutions[i].Lt = evolutions[i - 1].Lt.half_size();
            contrast_factor = contrast_factor * 0.75;
            debug!(
                "New image size: {}x{}, new contrast factor: {}",
                evolutions[i].Lt.width(),
                evolutions[i].Lt.height(),
                contrast_factor
            );
        } else {
            evolutions[i].Lt = evolutions[i - 1].Lt.clone();
        }
        evolutions[i].Lsmooth = gaussian_blur(&evolutions[i].Lt, 1.0f32, 5);
        let start = PreciseTime::now();
        evolutions[i].Lx = ops::derivatives::scharr(&evolutions[i].Lsmooth, true, false, 1);
        debug!(
            "Computing derivative Lx took {}.",
            start.to(PreciseTime::now())
        );
        evolutions[i].Ly = ops::derivatives::scharr(&evolutions[i].Lsmooth, false, true, 1);
        evolutions[i].Lflow = pm_g2(&evolutions[i].Lx, &evolutions[i].Ly, contrast_factor);
        evolutions[i].Lstep =
            GrayFloatImage::new(evolutions[i].Lt.width(), evolutions[i].Lt.height());
        for j in 0..evolutions[i].fed_tau_steps.len() {
            let step_size: f64 = evolutions[i].fed_tau_steps[j];
            let start = PreciseTime::now();
            ops::nonlinear_diffusion::calculate_step(&mut evolutions[i], step_size);
            debug!(
                "Used step size {}, took {}",
                step_size,
                start.to(PreciseTime::now())
            );
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
    let start = PreciseTime::now();
    create_nonlinear_scale_space(&mut evolutions, &float_image, options);
    debug!(
        "Creating scale space took {}.",
        start.to(PreciseTime::now())
    );
    let start = PreciseTime::now();
    ops::detector_response::detector_response(&mut evolutions, options);
    debug!(
        "Computing detector response took {}.",
        start.to(PreciseTime::now())
    );
    match std::env::var("AKAZE_SCALE_SPACE_DIR") {
        Ok(val) => {
            info!("Writing scale space; if you want to skip this step, undefine the env var AKAZE_SCALE_SPACE_DIR");
            let string_to_pass = val.to_string();
            std::fs::create_dir_all(&string_to_pass.clone()).unwrap();
            types::evolution::write_evolutions(
                &evolutions,
                std::path::Path::new(&string_to_pass.clone()).to_owned(),
            );
        }
        Err(_e) => {
            info!("Not writing scale space; do write scale space, define the env var AKAZE_SCALE_SPACE_DIR");
        }
    }
    warn!("TODO: finish");
    std::fs::write(output_features_path, "foo").unwrap(); // placeholder
}
