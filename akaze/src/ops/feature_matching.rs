use crate::types::feature_match::Match;
use crate::types::keypoint::Descriptor;
use time::PreciseTime;
/// Match two sets of keypoints and descriptors. The
/// Hamming distance is used to match the descriptor sets,
/// using a brute force algorithm.
///
/// # Arguments
/// * `descriptors_0` - The first set of descriptors.
/// * `descriptors_1` - The second set of desctiptors.
/// * `distance_threshold` - The distance threshold below which
///                         to accept a match.
/// * `lowes_ratio` - The ratio of descriptor 0 to descriptor 1
///                   above which a match is rejected.
///
/// Note: this implementation seems considerably slower than
/// OpenCV's implementation, and the only thing I can guess is
/// that the Hamming distance calculation is slower. It probably
/// warrants some further investigation.
///
/// # Return value
/// A vector of matches.
pub fn descriptor_match(
    descriptors_0: &[Descriptor],
    descriptors_1: &[Descriptor],
    distance_threshold: usize,
    lowes_ratio: f64,
) -> Vec<Match> {
    let start = PreciseTime::now();
    let mut output: Vec<Match> = vec![];
    let mut filtered_by_threshold = 0;
    let mut filtered_by_lowes = 0;
    let mut mean = 0.;
    let mut max = 0.;
    let mut min = std::f64::MAX;

    for (i, d0) in descriptors_0.iter().enumerate() {
        let mut min_distance = distance_threshold;
        let mut min_j = 0;
        let mut second_to_min_distance = min_distance;
        for (j, d1) in descriptors_1.iter().enumerate() {
            let distance = hamming_distance(d0, d1, second_to_min_distance);
            if distance < min_distance {
                second_to_min_distance = min_distance;
                min_distance = distance;
                min_j = j;
            } else if distance < second_to_min_distance {
                second_to_min_distance = distance;
            }
        }
        // Apply thresholding and Lowe's ratio.
        // We use the lowes ratio squared because if the hamming distance were treated
        // as an L2 norm like it is with other distance metrics, then the hamming distance
        // is effectively a squared L2 norm rather than an L1 norm.
        //
        // (d0) ^ (1/2) = lowes_ratio * d1 ^ (1/2)
        // ((d0) ^ (1/2) = lowes_ratio * d1 ^ (1/2)) ^ 2
        // d0 = lowes_ratio ^ 2 * d1
        // 
        // The last reduction step can be done because hamming distance is never negative.
        if (min_distance as f64) < (second_to_min_distance as f64) * lowes_ratio.powi(2) {
            if min_distance < (distance_threshold as usize) {
                output.push(Match {
                    index_0: i,
                    index_1: min_j,
                    distance: min_distance as f64,
                });
                mean += min_distance as f64;
                if (min_distance as f64) < min {
                    min = min_distance as f64;
                }
                if (min_distance as f64) > max {
                    max = min_distance as f64;
                }
            } else {
                filtered_by_threshold += 1;
            }
        } else {
            filtered_by_lowes += 1;
        }
    }
    mean /= (filtered_by_threshold + output.len()) as f64;
    debug!(
        "{} matches, {} filtered by threshold, {} filtered by lowes, dist min={}, mean={}, max={}, took {}.",
        output.len(),
        filtered_by_threshold,
        filtered_by_lowes,
        min,
        mean,
        max,
        start.to(PreciseTime::now()),
    );
    output
}

/// The Hamming distance between two descriptors.
/// Ex.
/// 0100100
/// 0100000
/// Hamming distance = 1: 1 bit position differs
///
/// # Arguments
/// * `d0` - The first descriptor.
/// * `d1` - The second descriptor.
/// * `bailout_distance` - If this distance is exceeded,
///    the calculation is immediately aborted and returned.
///    This can save a lot of time in searching for a minimum
///    distnce because we don't need to continue the distance
///    computation if the result would have been too large to
///    consider anyway.
/// # Return value
/// The Hamming distance
fn hamming_distance(d0: &Descriptor, d1: &Descriptor, bailout_distance: usize) -> usize {
    let mut distance = 0usize;
    for it in d0.vector.iter().zip(d1.vector.iter()) {
        let (x0, x1) = it;
        distance += (*x0 ^ *x1).count_ones() as usize;
        if distance > bailout_distance {
            break;
        }
    }
    distance as usize
}
