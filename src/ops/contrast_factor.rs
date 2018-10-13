extern crate image;
use ops;
use types::image::gaussian_blur;
use types::image::{GrayFloatImage, ImageFunctions};

/// This function computes a good empirical value for the k contrast factor
/// given an input image, the percentile (0-1), the gradient scale and the
/// number of bins in the histogram.
/// 
/// # Arguments
/// * `image` Input imagm
/// * `percentile` - Percentile of the image gradient histogram (0-1)
/// * `gradient_histogram_scale` - Scale for computing the image gradient histogram
/// * `nbins` - Number of histogram bins
/// # Return value
/// k contrast factor
#[allow(non_snake_case)]
pub fn compute_contrast_factor(
    image: &GrayFloatImage,
    percentile: f64,
    gradient_histogram_scale: f64,
    num_bins: usize,
) -> f64 {
    let mut num_points: f64 = 0.0;
    let mut hmax: f64 = 0.0;
    let mut histogram: Vec<f64> = vec![0f64; num_bins];
    let gaussian = gaussian_blur(image, gradient_histogram_scale as f32);
    let Lx = ops::derivatives::scharr(&gaussian, true, false, 1);
    let Ly = ops::derivatives::scharr(&gaussian, false, true, 1);
    for y in 1..(gaussian.height() - 1) {
        for x in 1..(gaussian.width() - 1) {
            let Lx: f64 = Lx.get(x, y) as f64;
            let Ly: f64 = Ly.get(x, y) as f64;
            let modg: f64 = f64::sqrt(Lx * Lx + Ly * Ly);
            if modg > hmax {
                hmax = modg;
            }
        }
    }
    for y in 1..(gaussian.height() - 1) {
        for x in 1..(gaussian.width() - 1) {
            let Lx: f64 = Lx.get(x, y) as f64;
            let Ly: f64 = Ly.get(x, y) as f64;
            let modg: f64 = f64::sqrt(Lx * Lx + Ly * Ly);
            if modg != 0.0 {
                let mut bin_number = f64::floor((num_bins as f64) * (modg / hmax)) as usize;
                if bin_number == num_bins {
                    bin_number = bin_number - 1;
                }
                histogram[bin_number] += 1f64;
                num_points += 1f64;
            }
        }
    }
    let threshold: usize = (num_points * percentile) as usize;
    let mut k: usize = 0;
    let mut num_elements: usize = 0;
    while num_elements < threshold && k < num_bins {
        num_elements = num_elements + histogram[k] as usize;
        k += 1;
    }
    debug!(
        "hmax: {}, threshold: {}, num_elements: {}",
        hmax, threshold, num_elements
    );
    let mut kperc: f64 = 0.03;
    if num_elements >= threshold {
        kperc = hmax * (k as f64) / (num_bins as f64);
    }
    kperc
}
