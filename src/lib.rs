
extern crate image;
extern crate imageproc;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::path::PathBuf;
use image::DynamicImage;
use image::GenericImage;
use image::GrayImage;
use image::ImageBuffer;
use image::Luma;
use image::Pixel;
type GrayFloatImage = ImageBuffer<Luma<f32>, Vec<f32>>;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    fn locate_test_data() -> PathBuf {
        let exe_path = ::std::env::current_exe().unwrap();
        let mut parent_path = exe_path.parent().unwrap().to_owned();
        parent_path.push("../../../test-data/akaze-test-data");
        parent_path
    }

    #[test]
    fn test_locate_data() {
        warn!(
            "Note: test data can be obtained from the akaze-test-data
            repository See README.md");
        let test_data_path = locate_test_data();
        let mut image_file_path = test_data_path;
        image_file_path.push("1.jpg");
        let metadata = ::std::fs::metadata(image_file_path).unwrap();
        assert!(metadata.is_file());
        let test_data_path = locate_test_data();
        let mut image_file_path = test_data_path;
        image_file_path.push("2.jpg");
        let metadata = ::std::fs::metadata(image_file_path).unwrap();
        assert!(metadata.is_file());
        let test_data_path = locate_test_data();
        let mut image_file_path = test_data_path;
        image_file_path.push("1-output");
        let metadata = ::std::fs::metadata(image_file_path).unwrap();
        assert!(metadata.is_dir());
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Config {
    /// Default number of sublevels per scale level
    num_sublevels: u32, 
    /// Maximum octave evolution of the image 2^sigma (coarsest scale sigma units)
    max_octave_evolution: u32,
    /// Base scale offset (sigma units)
    base_scale_offset: f64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            num_sublevels: 4,
            max_octave_evolution: 4,
            base_scale_offset: 1.6f64,
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
struct EvolutionStep {
    /// Evolution time
    etime: f64,
    /// Evolution sigma. For linear diffusion t = sigma^2 / 2
    esigma: f64,
    /// Image octave
    octave: u32,
    /// Image sublevel in each octave
    sublevel: u32,
    /// Integer sigma. For computing the feature detector responses
    sigma_size: u32,
    /// Evolution image
    Lt: GrayFloatImage,
    /// Smoothed image
    Lsmooth: GrayFloatImage,
    /// First order spatial derivative
    Lx: GrayFloatImage,
    /// First order spatial derivatives
    Ly: GrayFloatImage,
    /// Second order spatial derivative
    Lxx: GrayFloatImage,
    /// Second order spatial derivatives
    Lyy: GrayFloatImage,
    /// Second order spatial derivatives
    Lxy: GrayFloatImage,
    /// Diffusivity image
    Lflow: GrayFloatImage,
    /// Evolution step update
    Lstep: GrayFloatImage,
    /// Detector response
    Ldet: GrayFloatImage,
}

impl EvolutionStep {
    fn new(
        level_width: u32, level_height: u32, octave: u32, 
        sublevel: u32, options: Config) -> EvolutionStep {
        let esigma = options.base_scale_offset*f64::powf(2.0f64, (sublevel as f64)/( (options.num_sublevels  + octave) as f64));
        EvolutionStep {
            etime: 0.0f64,
            esigma: esigma,
            octave: octave,
            sublevel: sublevel,
            sigma_size: esigma.round() as u32,
            Lt: GrayFloatImage::new(level_width, level_height),
            Lsmooth: GrayFloatImage::new(level_width, level_height),
            Lx: GrayFloatImage::new(level_width, level_height),
            Ly: GrayFloatImage::new(level_width, level_height),
            Lxx: GrayFloatImage::new(level_width, level_height),
            Lyy: GrayFloatImage::new(level_width, level_height),
            Lxy: GrayFloatImage::new(level_width, level_height),
            Lflow: GrayFloatImage::new(level_width, level_height),
            Lstep: GrayFloatImage::new(level_width, level_height),
            Ldet: GrayFloatImage::new(level_width, level_height),
        }
    }
}

fn allocate_evolutions(width: u32, height: u32, options: Config) -> Vec<EvolutionStep> {
    let mut out_vec: Vec<EvolutionStep> = vec![];
    for i in 0..(options.max_octave_evolution-1u32) {
        let rfactor = 1.0f64/f64::powf(2.0f64, i as f64);
        let level_height = ( (height as f64)*rfactor) as u32;
        let level_width = ( (width as f64)*rfactor) as u32;
        // Smallest possible octave and allow one scale if the image is small
        if (level_width >= 80 && level_height >= 40) || i == 0 {
            for j in 0..options.num_sublevels {
                let evolution_step = EvolutionStep::new(level_width, level_height, i, j, options);
                out_vec.push(evolution_step);
            }
        } else {
            break;
        }
    }
    out_vec
}

fn create_unit_float_image(input_image: &DynamicImage) -> GrayFloatImage {
    let gray_image: GrayImage = input_image.to_luma();
    let mut output_image = GrayFloatImage::new(input_image.width(), input_image.height());
    for (x, y, gray_pixel) in gray_image.enumerate_pixels() {
        let pixel_value: u8 = gray_pixel.channels()[0];
        let scaled_float = (pixel_value as f32) * 1f32 / 255f32;
        output_image.put_pixel(x, y, image::Luma([scaled_float]));
    }
    output_image
}

fn create_nonlinear_scale_space(evolutions: &mut Vec<EvolutionStep>, image: &GrayFloatImage, _options: Config) {
  evolutions[0].Lt = image.clone();
  // TODO: blur with options.base_scale_offset
  evolutions[0].Lsmooth = evolutions[0].Lt.clone();
}

/// Extract features using the Akaze feature extractor.
/// 
/// # Arguments
/// `input_image_path` - the input image for which to extract features.
/// `output_features_path` - the output path to which to write an output JSON file.
/// `options: the options for the algorithm.`
/// 
pub fn extract_features(input_image_path: PathBuf, output_features_path: PathBuf, options: Config) {
    let input_image = image::open(input_image_path).unwrap();
    let float_image = create_unit_float_image(&input_image);
    info!("Loaded a {} x {} image", input_image.width(), input_image.height());
    let mut evolutions = allocate_evolutions(input_image.width(), input_image.height(), options);
    create_nonlinear_scale_space(&mut evolutions, &float_image, options);
    warn!("TODO: finish");
    std::fs::write(output_features_path, "foo").unwrap(); // placeholder
}
