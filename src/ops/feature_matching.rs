use types::feature_match::Match;
use types::keypoint::{Descriptor, Keypoint};

/// Match two sets of keypoints and descriptors. The
/// Hamming distance is used to determine the matches,
/// and a brute force algorithm is used to get the
/// best matches. Outliers are removed using Lowe's ratio.
/// `keypoints_0` the first set of keypoints.
/// `descriptors_0` The first set of descriptors.
/// `keypoints_1` - The second set of keypoints.
/// `keypoints_2` - The second set of desctiptors.
/// # Return value
/// A vector of matches.
pub fn descriptor_match(
    _keypoints_0: &Vec<Keypoint>,
    _descriptors_0: &Vec<Descriptor>,
    _keypoints_1: &Vec<Keypoint>,
    _descriptors_1: &Vec<Descriptor>,
) -> Vec<Match> {
    let output: Vec<Match> = vec![];
    warn!("TODO: descriptor_match");
    output
}
