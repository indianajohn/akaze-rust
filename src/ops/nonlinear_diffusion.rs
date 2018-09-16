
use types::evolution::EvolutionStep;
/// This function performs a scalar non-linear diffusion step
/// `Ld` Output image in the evolution
/// `c` Conductivity image. The function c is a scalar value that depends on the gradient norm
/// `Lstep` Previous image in the evolution
/// `step_size` The step size in time units
/// Forward Euler Scheme 3x3 stencil
/// dL_by_ds = d(c dL_by_dx)_by_dx + d(c dL_by_dy)_by_dy
#[allow(non_snake_case)]
pub fn calculate_step (
    _Ld: &mut EvolutionStep,
    _step_size: f64,
) {
    warn!("TODO: calculate_step in nonlinear_diffusion");
}