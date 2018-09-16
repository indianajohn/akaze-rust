use image::Pixel;
use types::evolution::EvolutionStep;
use types::image::GrayFloatImage;
use types::image::{gf, pf};
/// This function performs a scalar non-linear diffusion step
/// `Ld` Output image in the evolution
/// `c` Conductivity image. The function c is a scalar value that depends on the gradient norm
/// `Lstep` Previous image in the evolution
/// `step_size` The step size in time units
/// Forward Euler Scheme 3x3 stencil
/// dL_by_ds = d(c dL_by_dx)_by_dx + d(c dL_by_dy)_by_dy
#[allow(non_snake_case)]
pub fn calculate_step(evolution_step: &mut EvolutionStep, step_size: f64) {
    let Ld: &GrayFloatImage = &evolution_step.Lt;
    let c: &GrayFloatImage = &evolution_step.Lflow;
    let mut Lstep: &mut GrayFloatImage = &mut evolution_step.Lstep;

    // Diffusion all the image except borders
    for y in 1..(Lstep.height() - 1) {
        for x in 1..(Lstep.width() - 1) {
            let x_pos = (gf(c, x, y) + gf(c, x + 1, y)) * (gf(Ld, x + 1, y) - gf(Ld, x, y));
            let x_neg = (gf(c, x - 1, y) + gf(c, x + 1, y)) * (gf(Ld, x, y) - gf(Ld, x - 1, y));
            let y_pos = (gf(c, x, y) + gf(c, x, y + 1)) * (gf(Ld, x, y + 1) - gf(Ld, x, y));
            let y_neg = (gf(c, x, y - 1) + gf(c, x, y)) * (gf(Ld, x, y) - gf(Ld, x, y - 1));
            pf(
                &mut Lstep,
                x,
                y,
                0.5 * (step_size as f32) * (x_pos - x_neg + y_pos - y_neg),
            );
        }
    }

    // First row
    for x in 1..(Lstep.width() - 1) {
        let x_pos = (gf(c, x, 0) + gf(c, x + 1, 0)) * (gf(Ld, x + 1, 0) - gf(Ld, x, 0));
        let x_neg = (gf(c, x - 1, 0) + gf(c, x + 1, 0)) * (gf(Ld, x, 0) - gf(Ld, x - 1, 0));
        let y_pos = (gf(c, x, 0) + gf(c, x, 1)) * (gf(Ld, x, 1) - gf(Ld, x, 0));
        pf(
            &mut Lstep,
            x,
            0,
            0.5 * (step_size as f32) * (x_pos - x_neg + y_pos),
        );
    }
    {
        let x_pos = (gf(c, 0, 0) + gf(c, 1, 0)) * (gf(Ld, 1, 0) - gf(Ld, 0, 0));
        let y_pos = (gf(c, 0, 0) + gf(c, 0, 1)) * (gf(Ld, 0, 1) - gf(Ld, 0, 0));
        pf(&mut Lstep, 0, 0, 0.5 * (step_size as f32) * (x_pos + y_pos));
    }
    {
        let x = Lstep.width() - 1;
        let x_neg = (gf(c, x - 1, 0) + gf(c, x, 0)) * (gf(Ld, x, 0) - gf(Ld, x - 1, 0));
        let y_pos = (gf(c, x, 0) + gf(c, x, 1)) * (gf(Ld, x, 1) - gf(Ld, x, 0));
        pf(
            &mut Lstep,
            x,
            0,
            0.5 * (step_size as f32) * (-x_neg + y_pos),
        );
    }
}
