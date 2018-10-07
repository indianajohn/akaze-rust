use image::RgbImage;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use types::keypoint::Keypoint;
extern crate serde;
extern crate serde_json;

/// A match between a keypoint in one image and a keypoint
/// in another image.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Match {
    /// The index in the first image.
    pub index_0: usize,
    /// The index in the second image.
    pub index_1: usize,
    /// The distance between the two points in descriptor space.
    pub distance: f32,
}

/// Draw matches onto two images.
/// `input_image_0` The first image.
/// `input_image_1` The second image.
/// `keypoints_0` keypoints on the first image.
/// `keypoints_1` keypoints on the second image.
/// `matches` matches between the two sets of keypoints/images.
/// # Return value
/// An new RGB image with matches drawn.
pub fn draw_matches(
    input_image_0: &RgbImage,
    _input_image_1: &RgbImage,
    _keypoints_0: &Vec<Keypoint>,
    _keypoints_1: &Vec<Keypoint>,
    _matches: &Vec<Match>,
) -> RgbImage {
    warn!("TODO: draw_matches");
    input_image_0.clone()
}

/// Serialize matches to a file.
/// 'matches' - The matches to serialize.
/// `path` - Path to which to write.
pub fn serialize_to_file(matches: &Vec<Match>, path: PathBuf) {
    debug!("Writing results to {:?}", path);
    let mut file = File::create(path.clone()).unwrap();
    let extension = path.extension().unwrap();
    if extension == "json" {
        let serialized = serde_json::to_string(&matches).unwrap();
        file.write(serialized.as_bytes()).unwrap();
    } else if extension == "cbor" {
        let serialized = serde_cbor::to_vec(&matches).unwrap();
        file.write(&serialized[..]).unwrap();
    } else {
        // Default to JSON
        let serialized = serde_json::to_string(&matches).unwrap();
        file.write(serialized.as_bytes()).unwrap();
    }
}

/// Deserialize matches from a file.
/// 'path' - Path from which to read.
/// # Return value
/// The deserialized results.
pub fn deserialize_from_file(path: PathBuf) -> Vec<Match> {
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
