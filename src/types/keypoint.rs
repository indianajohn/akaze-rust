use image::{DynamicImage, Pixel, RgbImage};
use random;
use random::Source;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::u32;
use serde_json;
use serde_cbor;

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

fn random_color() -> (u8, u8, u8) {
    let mut source = random::default();
    (
        source.read::<u8>(),
        source.read::<u8>(),
        source.read::<u8>(),
    )
}

fn blend(p1: (u8, u8, u8), p2: (u8, u8, u8)) -> (u8, u8, u8) {
    (
        (((p1.0 as f32) + (p2.0 as f32)) / 2f32) as u8,
        (((p1.1 as f32) + (p2.1 as f32)) / 2f32) as u8,
        (((p1.2 as f32) + (p2.2 as f32)) / 2f32) as u8,
    )
}

/// Draw a circle to an image.
/// Values inside of the circle will be blended between their current color
/// value and the input.
///
/// `input_image` the image to draw on, directly mutated.
/// `point` the point at which to draw.
/// `rgb` The RGB value.
/// `radius` The maximum radius from the point to shade.
fn draw_circle(input_image: &mut RgbImage, point: (f32, f32), rgb: (u8, u8, u8), radius: f32) {
    for x in (point.0 as u32).saturating_sub(radius as u32)
        ..(point.0 as u32).saturating_add(radius as u32)
    {
        for y in (point.1 as u32).saturating_sub(radius as u32)
            ..(point.1 as u32).saturating_add(radius as u32)
        {
            let xy = (x as f32, y as f32);
            let delta_x = xy.0 - point.0;
            let delta_y = xy.1 - point.1;
            let radius_check = f32::sqrt(delta_x * delta_x + delta_y * delta_y);
            if radius_check <= radius {
                let pixel = input_image.get_pixel_mut(x, y);
                let rgb_point = (
                    pixel.channels()[0],
                    pixel.channels()[1],
                    pixel.channels()[2],
                );
                let color_to_set = blend(rgb, rgb_point);
                pixel.channels_mut()[0] = color_to_set.0;
                pixel.channels_mut()[1] = color_to_set.1;
                pixel.channels_mut()[2] = color_to_set.2;
            }
        }
    }
}

/// Draw keypoints onto an image
/// Keypoints of a random color will be drawn to the input image. The
/// points will be shaded between the existing pixel value and the
/// random color value.
/// `input_image` The image on which to draw.
/// `keypoints` a vector of keypoints to draw.
pub fn draw_keypoints_to_image(input_image: &mut RgbImage, keypoints: &Vec<Keypoint>) {
    for keypoint in keypoints.iter() {
        draw_circle(input_image, keypoint.point, random_color(), keypoint.size);
    }
}

/// Draw keypoints onto an image
/// Keypoints of a random color will be drawn to the input image. The
/// points will be shaded between the existing pixel value and the
/// random color value.
/// `input_image` The image on which to draw.
/// `keypoints` a vector of keypoints to draw.
/// # Return value
/// An new RGB image with keypoints drawn.
pub fn draw_keypoints(input_image: &DynamicImage, keypoints: &Vec<Keypoint>) -> RgbImage {
    let mut rgb_image = input_image.to_rgb();
    draw_keypoints_to_image(&mut rgb_image, keypoints);
    rgb_image
}

/// Serialize results to a file.
/// 'keypoints' - the keypoints detected from an image.
/// `descriptors` - The descriptors extracted from the keypoints. Will
///                 panic if the size of this vector is not equal to the
///                 size of the keypoints, or 0.
/// `path` - Path to which to write.
pub fn serialize_to_file(keypoints: &Vec<Keypoint>, descriptors: &Vec<Descriptor>, path: PathBuf) {
    debug!("Writing results to {:?}", path);
    let mut file = File::create(path.clone()).unwrap();
    let extension = path.extension().unwrap();
    let output = Results {
        keypoints: keypoints.clone(),
        descriptors: descriptors.clone(),
    };
    if extension == "json" {
        let serialized = serde_json::to_string(&output).unwrap();
        file.write(serialized.as_bytes()).unwrap();
    } else if extension == "cbor" {
        let serialized = serde_cbor::to_vec(&output).unwrap();
        file.write(&serialized[..]).unwrap();
    } else {
        // Default to JSON
        let serialized = serde_json::to_string(&output).unwrap();
        file.write(serialized.as_bytes()).unwrap();
    }
}

/// Serialize results to a file.
/// 'path' - Path from which to read.
/// # Return value
/// The deserialized results.
pub fn deserialize_from_file(path: PathBuf) -> Results {
    debug!("Reading results from {:?}", path);
    let mut file = File::open(path.clone()).unwrap();
    let extension = path.extension().unwrap();
    if extension == "json" {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        serde_json::from_str(&buffer).unwrap()
    } else if extension == "cbor" {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        serde_cbor::from_slice(&buffer[..]).unwrap()
    } else {
        // default to JSON
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        serde_json::from_str(&buffer).unwrap()
    }
}
