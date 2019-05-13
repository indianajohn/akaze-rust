use crate::types::image::{draw_circle, random_color};
use image::{DynamicImage, RgbImage};
use serde::{Deserialize, Serialize};

/// A point of interest in an image.
/// This pretty much follows from OpenCV conventions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Keypoint {
    /// The horizontal coordinate in a coordinate system is
    /// defined s.t. +x faces right and starts from the top
    /// of the image.
    /// the vertical coordinate in a coordinate system is defined
    /// s.t. +y faces toward the bottom of an image and starts
    /// from the left side of the image.
    pub point: (f32, f32),
    /// The magnitude of response from the detector.
    pub response: f32,

    /// The radius defining the extent of the keypoint, in pixel units
    pub size: f32,

    /// The level of scale space in which the keypoint was detected.
    pub octave: usize,

    /// A classification ID
    pub class_id: usize,

    /// The orientation angle
    pub angle: f32,
}

/// A feature descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    pub vector: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Results {
    pub keypoints: Vec<Keypoint>,
    pub descriptors: Vec<Descriptor>,
}

/// Draw keypoints onto an image
/// Keypoints of a random color will be drawn to the input image. The
/// points will be shaded between the existing pixel value and the
/// random color value.
///
/// # Arguments
/// * `input_image` - The image on which to draw.
/// * `keypoints` - A vector of keypoints to draw.
pub fn draw_keypoints_to_image(input_image: &mut RgbImage, keypoints: &[Keypoint]) {
    for keypoint in keypoints.iter() {
        draw_circle(input_image, keypoint.point, random_color(), keypoint.size);
    }
}

/// Draw keypoints onto an image
/// Keypoints of a random color will be drawn to the input image. The
/// points will be shaded between the existing pixel value and the
/// random color value.
///
/// # Arguments
/// * `input_image` - The image on which to draw.
/// * `keypoints` - A vector of keypoints to draw.
/// # Return value
/// An new RGB image with keypoints drawn.
pub fn draw_keypoints(input_image: &DynamicImage, keypoints: &[Keypoint]) -> RgbImage {
    let mut rgb_image = input_image.to_rgb();
    draw_keypoints_to_image(&mut rgb_image, keypoints);
    rgb_image
}
