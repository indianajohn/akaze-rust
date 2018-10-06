use nalgebra::{Matrix2, Vector2, LU};
use types::evolution::{Config, EvolutionStep};
use types::image::ImageFunctions;
use types::keypoint::Keypoint;

pub fn find_scale_space_extrema(
    evolutions: &mut Vec<EvolutionStep>,
    options: Config,
) -> Vec<Keypoint> {
    let mut keypoint_cache: Vec<Keypoint> = vec![];
    let smax = 10.0f32 * f32::sqrt(2.0f32);
    for (e_id, evolution) in evolutions.iter_mut().enumerate() {
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
        let mut x_p_i = x_p_iter.nth(w + 2).unwrap(); // 2, 1
        let mut y_m_iter = evolution.Ldet.buffer.iter();
        let mut y_m_i = y_m_iter.nth(1).unwrap(); // 1, 0
        let mut y_p_iter = evolution.Ldet.buffer.iter();
        let mut y_p_i = y_p_iter.nth(2 * w + 1).unwrap(); // 1, 2
                                                          // Iterate from 1,1 to the second-to-last pixel of the second-to-last row
        for i in (w + 1)..(evolution.Ldet.buffer.len() - w - 1) {
            let x = i % w;
            let y = i / w;
            // Apply detector threshold
            if x != 0 && x != w && // do nothing for border pixels we will encounter in the iteration range
                *x_i > (options.detector_threshold as f32) &&
                *x_i > *x_p_i &&
                *x_i > *x_m_i &&
                *x_i > *y_m_i &&
                *x_i > *y_p_i
            {
                let mut keypoint = Keypoint {
                    response: f32::abs(*x_i),
                    size: (evolution.esigma * options.derivative_factor) as f32,
                    octave: evolution.octave as usize,
                    class_id: e_id,
                    point: (x as f32, y as f32),
                };
                let ratio = f32::powf(2.0f32, evolution.octave as f32);
                let sigma_size = f32::round(keypoint.size / ratio);
                // Compare response with same and lower scale
                let mut id_repeated = 0;
                let mut is_repeated = false;
                let mut is_extremum = true;
                for (k, prev_keypoint) in keypoint_cache.iter().enumerate() {
                    if ((keypoint.class_id - 1) == prev_keypoint.class_id)
                        || (keypoint.class_id == prev_keypoint.class_id)
                    {
                        let dist = (keypoint.point.0 * ratio - prev_keypoint.point.0)
                            * (keypoint.point.0 * ratio - prev_keypoint.point.0)
                            + (keypoint.point.1 * ratio - prev_keypoint.point.1)
                                * (keypoint.point.1 * ratio - prev_keypoint.point.1);
                        if dist <= keypoint.size * keypoint.size {
                            if keypoint.response > prev_keypoint.response {
                                id_repeated = k;
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
                    let left_x = f32::round(keypoint.point.0 - smax * sigma_size) - 1f32;
                    let right_x = f32::round(keypoint.point.0 + smax * sigma_size) + 1f32;
                    let up_y = f32::round(keypoint.point.1 - smax * sigma_size) - 1f32;
                    let down_y = f32::round(keypoint.point.1 + smax * sigma_size) + 1f32;
                    let is_out = left_x < 0f32
                        || right_x >= (w as f32)
                        || up_y < 0f32
                        || down_y >= (h as f32);
                    if !is_out {
                        keypoint.point = (
                            keypoint.point.0 * ratio + 0.5f32 * (ratio - 1.0f32),
                            keypoint.point.1 * ratio + 0.5f32 * (ratio - 1.0f32),
                        );
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
    let mut output_keypoints: Vec<Keypoint> = vec![];
    for i in 0..keypoint_cache.len() {
        let mut is_repeated = false;
        let kp_i = keypoint_cache[i];
        for j in i..keypoint_cache.len() {
            // Compare response with the upper scale
            let kp_j = keypoint_cache[j];
            if (kp_i.class_id + 1) == kp_j.class_id {
                let dist = (kp_i.point.0 - kp_j.point.0) * (kp_i.point.0 - kp_j.point.0)
                    + (kp_i.point.1 - kp_j.point.1) * (kp_i.point.1 - kp_j.point.1);
                if dist <= kp_i.size * kp_i.size {
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
    in_keypoints: &Vec<Keypoint>,
    evolutions: &Vec<EvolutionStep>,
) -> Vec<Keypoint> {
    let mut result: Vec<Keypoint> = vec![];
    for keypoint in in_keypoints.iter() {
        let ratio = f32::powf(2.0f32, keypoint.octave as f32);
        let x = f32::round(keypoint.point.0 / ratio) as usize;
        let y = f32::round(keypoint.point.1 / ratio) as usize;
        let x_i = evolutions[keypoint.class_id].Ldet.get(x, y);
        let x_p = evolutions[keypoint.class_id].Ldet.get(x + 1, y);
        let x_m = evolutions[keypoint.class_id].Ldet.get(x - 1, y);
        let y_p = evolutions[keypoint.class_id].Ldet.get(x, y + 1);
        let y_m = evolutions[keypoint.class_id].Ldet.get(x, y - 1);
        // Derivative
        let d_x = 0.5f32 * (x_p - x_m);
        let d_y = 0.5f32 * (y_p - y_m);
        // Hessian
        let d_xx = x_p + x_m - 2f32 * x_i;
        let d_yy = y_p + y_m - 2f32 * x_i;
        let d_xy = 0.25f32 * (x_p + x_m) - 0.25f32 * (x_p + x_m);
        let a = Matrix2::new(d_xx, d_xy, d_xy, d_yy);
        let mut b = Vector2::new(-d_x, -d_y);
        let lu = LU::new(a.clone());
        lu.solve(&mut b);
        if f32::abs(b[0]) <= 1.0 && f32::abs(b[1]) <= 1.0 {
            let mut keypoint_clone = keypoint.clone();
            keypoint_clone.point = ((x as f32) + b[0], (y as f32) + b[1]);
            keypoint_clone.point = (
                keypoint_clone.point.0 * ratio + 0.5f32 * (ratio - 1f32),
                keypoint_clone.point.1 * ratio + 0.5f32 * (ratio - 1f32),
            );
            result.push(keypoint_clone);
        }
    }
    info!(
        "{}/{} remain after subpixel refinement.",
        result.len(),
        in_keypoints.len()
    );
    result
}

pub fn detect_keypoints(evolutions: &mut Vec<EvolutionStep>, options: Config) -> Vec<Keypoint> {
    let mut keypoints = find_scale_space_extrema(evolutions, options);
    keypoints = do_subpixel_refinement(&keypoints, &evolutions);
    keypoints
}
