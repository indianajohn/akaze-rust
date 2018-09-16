use types::image::GrayFloatImage;
use ops::fed_tau;

#[derive(Debug, Copy, Clone)]
pub struct Config {
    /// Default number of sublevels per scale level
    pub num_sublevels: u32,
    /// Maximum octave evolution of the image 2^sigma (coarsest scale sigma units)
    pub max_octave_evolution: u32,
    /// Base scale offset (sigma units)
    pub base_scale_offset: f64,
    /// The initial contrast factor parameter
    pub initial_contrast: f64,
    /// Percentile level for the contrast factor
    pub contrast_percentile: f64,
    /// Number of bins for the contrast factor histogram
    pub contrast_factor_num_bins: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            num_sublevels: 4,
            max_octave_evolution: 4,
            base_scale_offset: 1.6f64,
            initial_contrast: 0.001f64,
            contrast_percentile: 0.7f64,
            contrast_factor_num_bins: 300,
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct EvolutionStep {
    /// Evolution time
    pub etime: f64,
    /// Evolution sigma. For linear diffusion t = sigma^2 / 2
    pub esigma: f64,
    /// Image octave
    pub octave: u32,
    /// Image sublevel in each octave
    pub sublevel: u32,
    /// Integer sigma. For computing the feature detector responses
    pub sigma_size: u32,
    /// Evolution image
    pub Lt: GrayFloatImage,
    /// Smoothed image
    pub Lsmooth: GrayFloatImage,
    /// First order spatial derivative
    pub Lx: GrayFloatImage,
    /// First order spatial derivatives
    pub Ly: GrayFloatImage,
    /// Second order spatial derivative
    pub Lxx: GrayFloatImage,
    /// Second order spatial derivatives
    pub Lyy: GrayFloatImage,
    /// Second order spatial derivatives
    pub Lxy: GrayFloatImage,
    /// Diffusivity image
    pub Lflow: GrayFloatImage,
    /// Evolution step update
    pub Lstep: GrayFloatImage,
    /// Detector response
    pub Ldet: GrayFloatImage,
    /// fed_tau steps
    pub fed_tau_steps: Vec<f64>,
}

impl EvolutionStep {
    fn new(
        level_width: u32,
        level_height: u32,
        octave: u32,
        sublevel: u32,
        options: Config,
    ) -> EvolutionStep {
        let esigma = options.base_scale_offset * f64::powf(
            2.0f64,
            (sublevel as f64) / ((options.num_sublevels + octave) as f64),
        );
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
            fed_tau_steps: vec![],
        }
    }
}

pub fn allocate_evolutions(width: u32, height: u32, options: Config) -> Vec<EvolutionStep> {
    let mut out_vec: Vec<EvolutionStep> = vec![];
    for i in 0..(options.max_octave_evolution - 1u32) {
        let rfactor = 1.0f64 / f64::powf(2.0f64, i as f64);
        let level_height = ((height as f64) * rfactor) as u32;
        let level_width = ((width as f64) * rfactor) as u32;
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
    for i in 1..out_vec.len() {
        let ttime = out_vec[i].etime-out_vec[i-1].etime;
        out_vec[i].fed_tau_steps = fed_tau::fed_tau_by_process_time(
            ttime, 1, 0.25, true);
        debug!("{} steps in evolution {}.", out_vec[i].fed_tau_steps.len(), i);
    }
    out_vec
}
