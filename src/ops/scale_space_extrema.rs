use types::evolution::EvolutionStep;
use types::keypoint::Keypoint;

pub fn find_scale_space_extrema(
    _evolutions: &mut Vec<EvolutionStep>
) -> Vec<Keypoint> {
    let output_keypoints: Vec<Keypoint>  = vec![];
    warn!("TODO: finding scale space extrema.");
    output_keypoints
}

pub fn do_subpixel_refinement(
    in_keypoints: &mut Vec<Keypoint>
) -> Vec<Keypoint> {
    warn!("TODO: sub-pixel refinement.");
    in_keypoints.clone()
}

pub fn detect_keypoints(
    evolutions: &mut Vec<EvolutionStep>,
) -> Vec<Keypoint> {
    let mut keypoints = find_scale_space_extrema(evolutions);
    do_subpixel_refinement(&mut keypoints);
    keypoints
}