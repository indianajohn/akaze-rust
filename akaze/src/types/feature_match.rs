use crate::types::image::{draw_line, random_color};
use crate::types::keypoint::Keypoint;
use image::RgbImage;
use serde::{Serialize, Deserialize};

/// A match between a keypoint in one image and a keypoint
/// in another image.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Match {
    /// The index in the first image.
    pub index_0: usize,
    /// The index in the second image.
    pub index_1: usize,
    /// The distance between the two points in descriptor space.
    pub distance: f64,
}

fn map_pixel_in_1(combined_width: f32, x: f32, y: f32) -> (f32, f32) {
    (x + (combined_width / 2f32), y)
}

/// Draw matches onto two images.
///
/// # Arguments
/// * `input_image_0` - The first image.
/// * `input_image_1` - The second image.
/// * `keypoints_0` - Keypoints on the first image.
/// * `keypoints_1` - Keypoints on the second image.
/// * `matches` - Matches between the two sets of keypoints/images.
/// # Return value
/// An new RGB image with matches drawn.
pub fn draw_matches(
    input_image_0: &RgbImage,
    input_image_1: &RgbImage,
    keypoints_0: &[Keypoint],
    keypoints_1: &[Keypoint],
    matches: &[Match],
) -> RgbImage {
    debug!(
        "Writing match image for two images with sizes {}x{} and {}x{}.",
        input_image_0.width(),
        input_image_0.height(),
        input_image_1.width(),
        input_image_1.height()
    );
    // Get size of destination image
    let half_combined_width = u32::max(input_image_0.width(), input_image_1.width());
    let combined_width = 2 * half_combined_width;
    let combined_height = u32::max(input_image_0.height(), input_image_1.height());
    let mut combined_image = RgbImage::new(combined_width, combined_height);
    // first copy images
    for x in 0..input_image_0.width() {
        for y in 0..input_image_0.height() {
            *combined_image.get_pixel_mut(x, y) = *input_image_0.get_pixel(x, y);
        }
    }
    for x in 0..input_image_1.width() {
        for y in 0..input_image_1.height() {
            let (x_mapped, y_mapped) = map_pixel_in_1(combined_width as f32, x as f32, y as f32);
            *combined_image.get_pixel_mut(x_mapped as u32, y_mapped as u32) =
                *input_image_1.get_pixel(x, y);
        }
    }
    for match_i in matches.iter() {
        let keypoint_0 = keypoints_0[match_i.index_0];
        let keypoint_1 = keypoints_1[match_i.index_1];
        let pt_0 = keypoint_0.point;
        let pt_1 = map_pixel_in_1(
            combined_width as f32,
            keypoint_1.point.0,
            keypoint_1.point.1,
        );
        draw_line(
            &mut combined_image,
            pt_0,
            pt_1,
            random_color(),
            (combined_height as f32) / (500f32),
        );
    }
    combined_image
}
