use types::evolution::{Config, EvolutionStep};

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
    let mut output_descriptors: Vec<Descriptor> = vec![];
    warn!("TODO: mldb_binary_comparisons and mldb_fill_values");
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
    let mut output = Descriptor {
        vector: vec![],
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
            &values, &mut output.vector, val_count, &mut dpos);
    }
    output
}

fn mldb_fill_values(
    _values: &mut Vec<f32>, _sample_step: usize, _level: usize, 
    _xf: f32, _yf: f32, _co: f32, _si: f32, _scale: f32,
    _options: Config, _evolutions: &Vec<EvolutionStep>,
) {
}

fn mldb_binary_comparisons(
    _values: &Vec<f32>, _descriptor: &mut Vec<u8>,
    _count: usize, _dpos: &mut usize,
) {
}
