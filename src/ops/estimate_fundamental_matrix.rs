use nalgebra::{DMatrix, Matrix3, Vector3, SVD};
use random;
use random::Source;
use std::collections::HashSet;
use std::ops::{Index, IndexMut};
use crate::types::feature_match::Match;
use crate::types::keypoint::Keypoint;

/// Do singular value decomposition to estimate the fundamental matrix
/// given a set of 8 prospective inliers.
/// * `keypoints_0 ` - Keypoints in set 0
/// * `keypoints_1 ` - Keypoints in set 1
/// * `matches` - Set of matches referring to keypoints_0 and keypoints_1.
/// * `epsilon` - The epsilon to feed into SVD.
/// # Return value
/// Optionally, the fundamental matrix, a 3x3 matrix
pub fn estimate_fundamental_matrix(
    keypoints_0: &Vec<Keypoint>,
    keypoints_1: &Vec<Keypoint>,
    matches: &mut Vec<Match>,
    epsilon: f32,
) -> Option<Matrix3<f32>> {
    debug_assert!(matches.len() == 8);
    let mut a: DMatrix<f32> = DMatrix::zeros(8, 9);
    let mut b: DMatrix<f32> = DMatrix::zeros(8, 1);
    for (i, match_i) in matches.iter().enumerate() {
        *a.index_mut((i, 0)) =
            keypoints_0[match_i.index_0].point.0 * keypoints_1[match_i.index_1].point.0;
        *a.index_mut((i, 1)) =
            keypoints_0[match_i.index_0].point.0 * keypoints_1[match_i.index_1].point.1;
        *a.index_mut((i, 2)) = keypoints_0[match_i.index_0].point.0;
        *a.index_mut((i, 3)) =
            keypoints_0[match_i.index_0].point.1 * keypoints_1[match_i.index_1].point.0;
        *a.index_mut((i, 4)) =
            keypoints_0[match_i.index_0].point.1 * keypoints_1[match_i.index_1].point.1;
        *a.index_mut((i, 5)) = keypoints_0[match_i.index_0].point.1;
        *a.index_mut((i, 6)) = keypoints_1[match_i.index_1].point.0;
        *a.index_mut((i, 7)) = keypoints_1[match_i.index_1].point.1;
        *a.index_mut((i, 8)) = 1f32;
        *b.index_mut((i, 0)) = 0f32;
    }
    let svd = SVD::new(a, true, true);
    if svd.rank(epsilon) != 8 {
        None
    } else {
        let mut min_eigen_i = 0;
        let mut min_eigen = std::f32::MAX;
        for i in 0..8 {
            if svd.singular_values[i] < min_eigen {
                min_eigen_i = i;
                min_eigen = svd.singular_values[i];
            }
        }
        match svd.v_t {
            Some(v_t) => Some(Matrix3::new(
                *v_t.index((min_eigen_i, 0)),
                *v_t.index((min_eigen_i, 3)),
                *v_t.index((min_eigen_i, 6)),
                *v_t.index((min_eigen_i, 1)),
                *v_t.index((min_eigen_i, 4)),
                *v_t.index((min_eigen_i, 7)),
                *v_t.index((min_eigen_i, 2)),
                *v_t.index((min_eigen_i, 5)),
                *v_t.index((min_eigen_i, 8)),
            )),
            None => None,
        }
    }
}

/// Apply the fundamental matrix and return the error. to a pair of keypoints.
///
/// # Arguments
/// * `fund_mat` - the Fundamental Matrix
/// * `keypoint_0` - the keypoint in image plane l
/// * `keypoint_1` - the keypoint in image plane r
/// # Return value
/// The value of p_r.transpose()*F*p_l (= 0 defines epipolar lines)
fn evaluate_model(fund_mat: Matrix3<f32>, keypoint_0: &Keypoint, keypoint_1: &Keypoint) -> f32 {
    let p_r = Vector3::new(keypoint_1.point.0, keypoint_1.point.1, 1f32);
    let p_l = Vector3::new(keypoint_0.point.0, keypoint_0.point.1, 1f32);
    (p_r.transpose() * fund_mat * p_l).norm()
}

/// Remove outliers using RANSAC.
/// Contains RANSAC implementation.
///
/// # Arguments
/// * `keypoints_0` - Tirst set of keypoints
/// * `keypoints_1` - Second set of keypoints
/// * `matches` - Candidate matches
/// * `num_trials` - Maximum number of RANSAC iterations
/// * `epsilon_model` - epsilon used when solving SVD
/// * `epsilon_inliers` - Maximum error to accept an inlier.
///
/// # Return value
/// The inlier matches. If no model was found, the size
/// of the vector will be 0.
pub fn remove_outliers(
    keypoints_0: &Vec<Keypoint>,
    keypoints_1: &Vec<Keypoint>,
    matches: &Vec<Match>,
    num_trials: usize,
    epsilon_model: f32,
    epsilon_inlier: f32,
) -> Vec<Match> {
    if matches.len() < 8 {
        warn!("Not enough points to do RANSAC.");
        return matches.clone();
    } else {
        debug!("Removing outliers with RANSAC using fundamental matrix model.");
    }
    let mut max_inlier_count = 0;
    let mut final_model: Matrix3<f32> = Matrix3::zeros();
    for _ in 0..num_trials {
        // Pick the points for the model
        let mut set = HashSet::new();
        let mut source = random::default();
        while set.len() < 8 {
            set.insert(source.read::<usize>() % matches.len());
        }
        let mut model_matches: Vec<Match> = vec![];
        for j in set {
            model_matches.push(matches[j]);
        }

        // Get a model
        match estimate_fundamental_matrix(
            &keypoints_0,
            &keypoints_1,
            &mut model_matches,
            epsilon_model,
        ) {
            Some(model) => {
                let mut inlier_count = 0;
                for match_i in matches {
                    let error_i = evaluate_model(
                        model,
                        &keypoints_0[match_i.index_0],
                        &keypoints_1[match_i.index_1],
                    );
                    if error_i < epsilon_inlier {
                        inlier_count += 1;
                    }
                }
                if inlier_count > max_inlier_count {
                    max_inlier_count = inlier_count;
                    final_model = model;
                }
            }
            None => {}
        }
    }

    // Calculate final inlier set
    let mut inliers: Vec<Match> = vec![];
    for match_i in matches {
        let error_i = evaluate_model(
            final_model,
            &keypoints_0[match_i.index_0],
            &keypoints_1[match_i.index_1],
        );
        if error_i < epsilon_inlier {
            inliers.push(*match_i);
        }
    }
    inliers
}
