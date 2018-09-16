extern crate image;
use image::Pixel;
use ops;
use types;
use types::image::GrayFloatImage;
#[allow(non_snake_case)]

/// This function computes a good empirical value for the k contrast factor
/// given an input image, the percentile (0-1), the gradient scale and the
/// number of bins in the histogram.
/// `image` Input imagm
/// `percentile` Percentile of the image gradient histogram (0-1)
/// `gradient_histogram_scale` Scale for computing the image gradient histogram
/// `nbins` nbins Number of histogram bins
/// # Return value
/// k contrast factor
pub fn compute_contrast_factor(
    image: &GrayFloatImage,
    percentile: f64,
    gradient_histogram_scale: f64,
    num_bins: usize,
) -> f64 {
    let mut bin_number: usize = 0;
    let mut num_elements: usize = 0;
    let mut num_points: f64 = 0.0;
    let mut hmax: f64 = 0.0;
    let mut histogram: Vec<f64> = vec![0f64; num_bins];
    let gaussian = image::imageops::blur(image, gradient_histogram_scale as f32);
    let Lx = ops::derivatives::scharr(&gaussian, true, false);
    let Ly = ops::derivatives::scharr(&gaussian, false, true);
    for y in 1..(gaussian.height() - 1) {
        for x in 1..(gaussian.width() - 1) {
            let Lx: f64 = Lx.get_pixel(x, y).channels()[0] as f64;
            let Ly: f64 = Ly.get_pixel(x, y).channels()[0] as f64;
            let modg: f64 = f64::sqrt(Lx * Lx + Ly * Ly);
            if modg > hmax {
                hmax = modg;
            }
        }
    }
    for y in 1..(gaussian.height() - 1) {
        for x in 1..(gaussian.width() - 1) {
            let Lx: f64 = Lx.get_pixel(x, y).channels()[0] as f64;
            let Ly: f64 = Ly.get_pixel(x, y).channels()[0] as f64;
            let modg: f64 = f64::sqrt(Lx * Lx + Ly * Ly);
            if modg != 0.0 {
                bin_number = f64::floor((num_bins as f64) * (modg / hmax)) as usize;
            }
            if bin_number == num_bins {
                bin_number = bin_number - 1;
            }
            histogram[bin_number] += 1f64;
            num_points += 1f64;
        }
    }
    let threshold: usize = (num_points * percentile) as usize;
    let mut k: usize = 0;
    while k < threshold && k < num_bins {
        num_elements = num_elements + histogram[k] as usize;
        k += 1;
    }
    let mut kperc: f64 = 0.03;
    if num_elements >= threshold {
        kperc = hmax * (k as f64) / (num_bins as f64);
    }
    kperc
}
