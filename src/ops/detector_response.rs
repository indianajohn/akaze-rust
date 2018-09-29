
use types::evolution::EvolutionStep;
use num_cpus;
use scoped_threadpool::Pool;
use types::evolution::Config;
use ops::derivatives;

fn compute_multiscale_derivatives_for_evolution(
    evolution: &mut EvolutionStep,
    sigma_size: u32) {
    evolution.Lx = derivatives::scharr(&evolution.Lsmooth, true, false, sigma_size);
    evolution.Ly = derivatives::scharr(&evolution.Lsmooth, false, true, sigma_size);
    evolution.Lxx = derivatives::scharr(&evolution.Lx, true, false, sigma_size);
    evolution.Lxy = derivatives::scharr(&evolution.Lx, false, true, sigma_size);
}

fn compute_multiscale_derivatives(evolutions: &mut Vec<EvolutionStep>, options: Config) {
    let cpu_count = num_cpus::get();
    let mut pool = Pool::new(cpu_count as u32);
    pool.scoped(|scoped| {
        for evolution in evolutions.iter_mut() {
            scoped.execute(move|| {
                let ratio = f64::powf(2.0f64, evolution.octave as f64);
                let sigma_size = f64::round(evolution.esigma*options.derivative_factor/ratio) as u32;
                compute_multiscale_derivatives_for_evolution(evolution, sigma_size);
            });
        }
    });
}

pub fn detector_response(
    evolutions: &mut Vec<EvolutionStep>,
    options: Config) {
    compute_multiscale_derivatives(evolutions, options);
}