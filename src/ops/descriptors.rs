use types::evolution::{Config, EvolutionStep};
use types::image::ImageFunctions;

use types::keypoint::{Descriptor, Keypoint};

/// Extract descriptors from keypoints/an evolution
/// `evolutions` - the nonlinear scale space
/// `keypoints` - the keypoints detected.
/// `options` - The options of the nonlinear scale space.
/// # Return value
/// A vector of descriptors.
pub fn extract_descriptors(
    evolutions: &Vec<EvolutionStep>,
    keypoints: &Vec<Keypoint>,
    options: Config,
) -> Vec<Descriptor> {
    //int t = (6+36+120)*options_.descriptor_channels
    //desc = cv::Mat::zeros(kpts.size(), ceil(t/8.), CV_8UC1);
    let mut output_descriptors: Vec<Descriptor> = vec![];
    for keypoint in keypoints {
        output_descriptors.push(get_mldb_descriptor(keypoint, evolutions, options));
    }
    output_descriptors
}

fn get_mldb_descriptor(
    keypoint: &Keypoint,
    evolutions: &Vec<EvolutionStep>,
    options: Config,
) -> Descriptor {
    let t = (6usize+36usize+120usize)*options.descriptor_channels;
    let mut output = Descriptor {
        // 486 bit descriptor
        vector: vec![0u8; t],
    };
    let max_channels = 3usize;
    debug_assert!(options.descriptor_channels <= max_channels);
    let mut values: Vec<f32> = vec![0f32; (16*max_channels) as usize];
    let size_mult = [1.0f32, 2.0f32/3.0f32, 1.0f32/2.0f32];
    let ratio = (1u32 << keypoint.octave) as f32;
    let scale = f32::round(0.5f32 * (keypoint.size as f32) / ratio);
    let xf = keypoint.point.0 / ratio;
    let yf = keypoint.point.1 / ratio;
    let co = f32::cos(keypoint.angle);
    let si = f32::sin(keypoint.angle);
    let mut dpos = 0usize;
    let pattern_size = options.descriptor_pattern_size as f32;
    for lvl in 0..3 {
        let val_count = (lvl + 2usize) * (lvl + 2usize);
        let sample_size = f32::ceil(pattern_size * size_mult[lvl]) as usize;
        mldb_fill_values(
            &mut values, sample_size, keypoint.class_id, 
            xf, yf, co, si, scale, options, &evolutions);
        mldb_binary_comparisons(
            &values, &mut output.vector, val_count, 
            &mut dpos, options.descriptor_channels);
    }
    output
}

fn mldb_fill_values(
    values: &mut Vec<f32>, sample_step: usize, level: usize, 
    xf: f32, yf: f32, co: f32, si: f32, scale: f32,
    options: Config, evolutions: &Vec<EvolutionStep>,
) {
    let pattern_size = options.descriptor_pattern_size;
    let nr_channels = options.descriptor_channels;
    let mut valuepos = 0;
    for i in (-(pattern_size as i32)..(pattern_size as i32)).step_by(sample_step) {
        for j in (-(pattern_size as i32)..(pattern_size as i32)).step_by(sample_step) {
            let mut di = 0f32;
            let mut dx = 0f32;
            let mut dy = 0f32;
            let mut nsamples = 0usize;
            for k in i..(i + (sample_step as i32)) {
                for l in i..(j + (sample_step as i32)) {
                    let sample_y = yf + ((l as f32) * co * scale + (k as f32) * si * scale);
                    let sample_x = xf + ((-l as f32) * si * scale + (k as f32) * co * scale);
                    let y1 = f32::round(sample_y) as usize;
                    let x1 = f32::round(sample_x) as usize;
                    let ri = evolutions[level].Lt.get(x1, y1);
                    di += ri;
                    if nr_channels > 1 {
                        let rx = evolutions[level].Lx.get(x1, y1);
                        let ry = evolutions[level].Ly.get(x1, y1);
                        if nr_channels == 2 {
                            dx += f32::sqrt(rx*rx + ry*ry);
                        } else {
                            let rry = rx*co + ry*si;
                            let rrx = -rx*si + ry*co;
                            dx += rrx;
                            dy += rry;
                        }
                    }
                    nsamples += 1;
                }
            }
            di /= nsamples as f32;
            dx /= nsamples as f32;
            dy /= nsamples as f32;

            values[valuepos] = di;

            if nr_channels > 1 {
                values[valuepos + 1] = dx;
            }
            if nr_channels > 2 {
                values[valuepos + 2] = dy;
            }
            valuepos += nr_channels;
        }
    }
}

fn mldb_binary_comparisons(
    values: &Vec<f32>, descriptor: &mut Vec<u8>,
    count: usize, dpos: &mut usize, nr_channels: usize,
) {
    for pos in 0..nr_channels {
        for i in 0..count {
            let ival = values[nr_channels * i + pos];
            for j in (i + 1)..count {
                let mut res = 0u8;
                if ival > values[nr_channels * j + pos] {
                    res = 1u8;
                }
                descriptor[*dpos >> 3usize] |= res << (*dpos & 7);
                *dpos += 1usize;
            }
        }
    }
}
