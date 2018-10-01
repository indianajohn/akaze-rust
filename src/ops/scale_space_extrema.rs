use types::evolution::{EvolutionStep, Config};
use types::keypoint::Keypoint;
use types::image::ImageFunctions;

pub fn find_scale_space_extrema(
    evolutions: &mut Vec<EvolutionStep>,
    options: Config
) -> Vec<Keypoint> {
    let mut keypoint_cache: Vec<Keypoint> = vec![];
    let smax = 10.0f32*f32::sqrt(2.0f32);
    for evolution in evolutions {
        let w = evolution.Ldet.width();
        let h = evolution.Ldet.height();
        // maintain 5 iterators, one for the current pixel and one
        // for each cardinal pixel. Iterate through all non-border
        // pixels
        let mut x_m_iter = evolution.Ldet.buffer.iter();
        let mut x_m_i = x_m_iter.nth(w).unwrap(); // 0, 1
        let mut x_iter = evolution.Ldet.buffer.iter();
        let mut x_i = x_iter.nth(w + 1).unwrap(); // 1, 1
        let mut x_p_iter = evolution.Ldet.buffer.iter();
        let mut x_p_i = x_p_iter.nth(w).unwrap(); // 2, 1
        let mut y_m_iter = evolution.Ldet.buffer.iter();
        let mut y_m_i = y_m_iter.nth(1).unwrap(); // 1, 0
        let mut y_p_iter = evolution.Ldet.buffer.iter();
        let mut y_p_i = y_p_iter.nth(1).unwrap(); // 1, 2
        // Iterate from 1,1 to the second-to-last pixel of the second-to-last row
        for i in (h + 1)..(evolution.Ldet.buffer.len() - h - 1) {
            let x = i % w;
            let y = i / w;
            // do nothing for border pixels we will encounter in the iteration range
            if x == 0 || x == w {
                continue;
            }

            // Apply detector threshold
            if 
                f32::abs(*x_i) > (options.detector_threshold as f32) &&
                f32::abs(*x_i) > f32::abs(*x_p_i) &&
                f32::abs(*x_i) > f32::abs(*x_m_i) &&
                f32::abs(*x_i) > f32::abs(*y_m_i) &&
                f32::abs(*x_i) > f32::abs(*y_p_i)
            {
                let mut keypoint = Keypoint{
                    response: f32::abs(*x_i),
                    size: (evolution.esigma * options.derivative_factor) as f32,
                    octave: evolution.octave as usize,
                    class_id: (y * w + x),
                    point: (x as f32, y as f32),
                };
                let ratio =  f32::powf(2.0f32, evolution.octave as f32);
                let sigma_size = f32::round(keypoint.size / ratio);
                // Compare response with same and lower scale
                let mut id_repeated = 0;
                let mut is_repeated = false;
                let mut is_extremum = true;
                for (i, prev_keypoint) in keypoint_cache.iter().enumerate() {
                    if ((keypoint.class_id - 1) == prev_keypoint.class_id) || (keypoint.class_id == prev_keypoint.class_id) {
                        let dist = 
                            (keypoint.point.0 * ratio - prev_keypoint.point.0) * (keypoint.point.0 * ratio - prev_keypoint.point.0) +
                            (keypoint.point.1 * ratio - prev_keypoint.point.1) * (keypoint.point.1 * ratio - prev_keypoint.point.1);
                        if dist <= keypoint.size * keypoint.size {
                            if keypoint.response > prev_keypoint.response {
                                id_repeated = i;
                                is_repeated = true;
                            } else {
                                is_extremum = false;
                            }
                            break;
                        }
                    }
                }
                // Check bounds
                if is_extremum {
                    // Check that the point is under the image limits for the descriptor computation
                    let left_x = f32::round(keypoint.point.0 - smax*sigma_size) - 1f32;
                    let right_x = f32::round(keypoint.point.0 + smax*sigma_size) + 1f32;
                    let up_y = f32::round(keypoint.point.1 - smax*sigma_size) - 1f32;
                    let down_y = f32::round(keypoint.point.1 + smax*sigma_size) + 1f32;
                    let is_out = left_x < 0f32 || right_x >= (w as f32) || up_y < 0f32 || down_y >= (h as f32);
                    if !is_out {
                        keypoint.point = (
                            keypoint.point.0 * ratio + 0.5f32 * (ratio - 1.0f32), 
                            keypoint.point.1 * ratio + 0.5f32 * (ratio - 1.0f32));
                        if !is_repeated {
                            keypoint_cache.push(keypoint);
                        } else {
                            keypoint_cache[id_repeated] = keypoint;
                        }
                    }
                }
            }

            // increment iterators
            x_i = x_iter.next().unwrap();
            x_m_i = x_m_iter.next().unwrap();
            x_p_i = x_p_iter.next().unwrap();
            y_m_i = y_m_iter.next().unwrap();
            y_p_i = y_p_iter.next().unwrap();

        }
    }
    // Now filter points with the upper scale level
    let mut output_keypoints: Vec<Keypoint>  = vec![];
    for i in 0..keypoint_cache.len() {
        let mut is_repeated = false;
        let kp_i = keypoint_cache[i];
        for j in i..keypoint_cache.len() {
            // Compare response with the upper scale
            let kp_j = keypoint_cache[j];
            if (kp_i.class_id + 1) == kp_j.class_id {
                let dist = 
                    (kp_i.point.0 - kp_j.point.0) * (kp_i.point.0 - kp_j.point.0) +
                    (kp_i.point.1 - kp_j.point.1) * (kp_i.point.1 - kp_j.point.1);
                if dist <= kp_i.size*kp_i.size {
                    is_repeated = true;
                    break;
                }
            }
        }
        if !is_repeated {
            output_keypoints.push(kp_i);
        }
    }
    info!("Extracted {} scale space extrema.", output_keypoints.len());
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
    options: Config,
) -> Vec<Keypoint> {
    let mut keypoints = find_scale_space_extrema(evolutions, options);
    do_subpixel_refinement(&mut keypoints);
    keypoints
}