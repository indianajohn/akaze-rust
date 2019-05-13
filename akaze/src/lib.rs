#[macro_use]
extern crate log;

use image::GenericImageView;
use std::path::PathBuf;
use time::PreciseTime;

pub mod ops;
pub mod types;
use ops::estimate_fundamental_matrix::remove_outliers;
use types::evolution::{Config, EvolutionStep};
use types::feature_match::Match;
use types::image::{gaussian_blur, GrayFloatImage, ImageFunctions};
use types::keypoint::{Descriptor, Keypoint};

/// This function computes the Perona and Malik conductivity coefficient g2
/// g2 = 1 / (1 + dL^2 / k^2)
///
/// # Arguments
/// * `Lx` - First order image derivative in X-direction (horizontal)
/// * `Ly` - First order image derivative in Y-direction (vertical)
/// * `k` - Contrast factor parameter
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
            let Lx_pixel: f64 = f64::from(Lx.get(x, y));
            let Ly_pixel: f64 = f64::from(Ly.get(x, y));
            let dst_pixel: f64 =
                1.0f64 / (1.0f64 + inverse_k * (Lx_pixel * Lx_pixel + Ly_pixel * Ly_pixel));
            dst.put(x, y, dst_pixel as f32);
        }
    }
    dst
}

/// A nonlinear scale space performs selective blurring to preserve edges.
///
/// # Arguments
/// * `evolutions` - The output scale space.
/// * `image` - The input image.
/// * `options` - The options to use.
fn create_nonlinear_scale_space(
    evolutions: &mut Vec<EvolutionStep>,
    image: &GrayFloatImage,
    options: Config,
) {
    debug!("Creating first evolution.");
    let start = PreciseTime::now();
    evolutions[0].Lt = gaussian_blur(image, options.base_scale_offset as f32);
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
        debug!("Creating evolution {}.", i);
        if evolutions[i].octave > evolutions[i - 1].octave {
            let start = PreciseTime::now();
            evolutions[i].Lt = evolutions[i - 1].Lt.half_size();
            debug!("Half-sizing took {}", start.to(PreciseTime::now()));
            contrast_factor *= 0.75;
            debug!(
                "New image size: {}x{}, new contrast factor: {}",
                evolutions[i].Lt.width(),
                evolutions[i].Lt.height(),
                contrast_factor
            );
        } else {
            evolutions[i].Lt = evolutions[i - 1].Lt.clone();
        }
        let start = PreciseTime::now();
        evolutions[i].Lsmooth = gaussian_blur(&evolutions[i].Lt, 1.0f32);
        debug!("Gaussian blur took {}.", start.to(PreciseTime::now()));
        let start = PreciseTime::now();
        evolutions[i].Lx = ops::derivatives::scharr(&evolutions[i].Lsmooth, true, false, 1);
        debug!(
            "Computing derivative Lx took {}.",
            start.to(PreciseTime::now())
        );
        evolutions[i].Ly = ops::derivatives::scharr(&evolutions[i].Lsmooth, false, true, 1);
        let start = PreciseTime::now();
        evolutions[i].Lflow = pm_g2(&evolutions[i].Lx, &evolutions[i].Ly, contrast_factor);
        debug!("Lflow took {}", start.to(PreciseTime::now()));
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

/// Find image keypoints using the Akaze feature extractor.
///
/// # Arguments
/// * `input_image` - An image from which to extract features.
/// * `options` the options for the algorithm.
/// # Return Value
/// The resulting keypoints.
///
fn find_image_keypoints(evolutions: &mut Vec<EvolutionStep>, options: Config) -> Vec<Keypoint> {
    let start = PreciseTime::now();
    ops::detector_response::detector_response(evolutions, options);
    debug!(
        "Computing detector response took {}.",
        start.to(PreciseTime::now())
    );
    ops::scale_space_extrema::detect_keypoints(evolutions, options)
}

/// Extract features using the Akaze feature extractor.
///
/// This performs all operations end-to-end. The client might be only interested
/// in certain portions of the process, all of which are exposed in public functions,
/// but this function can document how the various parts fit together.
///
/// # Arguments
/// * `input_image_path` - The input image for which to extract features.
/// * `output_features_path` - The output path to which to write an output JSON file.
/// * `options` The options for the algorithm.
///
/// # Return value
/// * The evolutions of the process. Can be used for further analysis or visualization, or ignored.
/// * The keypoints at which features occur.
/// * The descriptors that were computed.
///
/// # Examples
/// ```no_run
/// extern crate akaze;
/// use std::path::Path;
/// let options = akaze::types::evolution::Config::default();
/// let (_evolutions, keypoints, descriptors) =
///     akaze::extract_features(
///       Path::new("test-data/1.jpg").to_owned(),
///       options);
/// ```
///
pub fn extract_features(
    input_image_path: PathBuf,
    options: Config,
) -> (Vec<EvolutionStep>, Vec<Keypoint>, Vec<Descriptor>) {
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
    let keypoints = find_image_keypoints(&mut evolutions, options);
    let start = PreciseTime::now();
    let descriptors = ops::descriptors::extract_descriptors(&evolutions, &keypoints, options);
    debug!(
        "Computing descriptors took {}.",
        start.to(PreciseTime::now())
    );
    (evolutions, keypoints, descriptors)
}

/// Match two sets of keypoints and descriptors. The
/// Hamming distance is used to match the descriptor sets,
/// using a brute force algorithm. Then, geometric verification
/// is performed using RANSAC with the Fundamental matrix and
/// 8-point algorithm.
///
/// There are some variations on all of the above - for example,
/// we could consider using a cascade hashing matching process -
/// but this is sufficient for validation of this repository. Any
/// further optimization is out of scope for this repository.
///
/// # Arguments
/// * `keypoints_0` - The first set of keypoints
/// * `descriptors_0` - The first set of descriptors
/// * `keypoints_1` - The first set of keypoints
/// * `descriptors_1` - The second set of desctiptors
/// * `lowes_ratio` - The ratio between the best and second-best match L2 norm required
/// * `ransac_trials` - The number of trials to run RANSAC
/// * `ransac_epsilon_inliers` - The maximum error to accept for an inlier
/// 
/// For a high number of matches with some error, choose:
/// * `lowes_ratio` - `0.86`
/// * `ransac_trials` - `1000`
/// * `ransac_epsilon_inliers` - `3.0`
/// 
/// If you have different performance constraints or input imagery,
/// experiment with the parameters. If you have more points, you may need to
/// lower the `lowes_ratio` to, at most, about `0.75`. You can also decrease
/// `ransac_epsilon_inliers` to achieve higher accuracy, but you will loose
/// some inliers. It seems that lowering the `lowes_ratio` is more effective
/// to reduce error with more features than modifying `ransac_epsilon_inliers`.
/// This might help as a starting point, but you should test the recall rate
/// in your own benchmarks with the data you have because every dataset is
/// different.
///
/// # Return value
/// A vector of matches.
///
/// # Examples
/// ```no_run
/// extern crate akaze;
/// use std::path::Path;
/// let options = akaze::types::evolution::Config::default();
/// let (_evolutions_0, keypoints_0, descriptors_0) =
///     akaze::extract_features(
///       Path::new("test-data/1.jpg").to_owned(),
///       options);
///
/// let (_evolutions_1, keypoints_1, descriptors_1) =
///     akaze::extract_features(
///       Path::new("test-data/2.jpg").to_owned(),
///       options);
/// let matches = akaze::match_features(&keypoints_0, &descriptors_0, &keypoints_1, &descriptors_1, 0.8, 10000, 0.25);
/// println!("Got {} matches.", matches.len());
/// ```
///
pub fn match_features(
    keypoints_0: &[Keypoint],
    descriptors_0: &[Descriptor],
    keypoints_1: &[Keypoint],
    descriptors_1: &[Descriptor],
    lowes_ratio: f64,
    ransac_trials: usize,
    ransac_epsilon_inliers: f32,
) -> Vec<Match> {
    let output = ops::feature_matching::descriptor_match(
        descriptors_0,
        descriptors_1,
        10000,
        lowes_ratio,
    );
    remove_outliers(
        &keypoints_0,
        &keypoints_1,
        &output,
        ransac_trials,
        0.05,
        ransac_epsilon_inliers,
    )
}
