use types::evolution::{EvolutionStep, Config};
use types::keypoint::{Keypoint, Descriptor};

/// Extract descriptors from keypoints/an evolution
/// `evolutions` - the nonlinear scale space
/// `keypoints` - the keypoints detected.
/// `options` - The options of the nonlinear scale space.
/// # Return value
/// A vector of descriptors.
pub fn extract_descriptors(
    _evolutions: &Vec<EvolutionStep>,
    _keypoints: &Vec<Keypoint>,
    _options: Config,
) -> Vec<Descriptor> {
    let output_descriptors: Vec<Descriptor> = vec![];
    warn!("TODO: extract descriptors.");
    output_descriptors
}