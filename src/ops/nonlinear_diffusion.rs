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
            let x_pos = eval(c, Ld, x, y, 1, 0);
            let y_pos = eval(c, Ld, x, y, 0, 1);
            let x_neg = eval(c, Ld, x, y, -1, 0);
            let y_neg = eval(c, Ld, x, y, 0, -1);
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
        let y = 0;
        let x_pos = eval(c, Ld, x, y, 1, 0);
        let y_pos = eval(c, Ld, x, y, 0, 1);
        let x_neg = eval(c, Ld, x, y, -1, 0);
        pf(
            &mut Lstep,
            x,
            y,
            0.5 * (step_size as f32) * (x_pos - x_neg + y_pos),
        );
    }
    {
        let x = 0;
        let y = 0;
        let x_pos = eval(c, Ld, x, y, 1, 0);
        let y_pos = eval(c, Ld, x, y, 0, 1);
        pf(&mut Lstep, x, y, 0.5 * (step_size as f32) * (x_pos + y_pos));
    }
    {
        let x = Lstep.width() - 1;
        let y = 0;
        let y_pos = eval(c, Ld, x, y, 0, 1);
        let x_neg = eval(c, Ld, x, y, -1, 0);
        pf(
            &mut Lstep,
            x,
            y,
            0.5 * (step_size as f32) * (-x_neg + y_pos),
        );
    }
    // Last row
    let y = Lstep.height() - 1;
    for x in 1..(Lstep.width() - 1) {
        let x_pos = eval(c, Ld, x, y, 1, 0);
        let y_pos = eval(c, Ld, x, y, 0, -1);
        let x_neg = eval(c, Ld, x, y, -1, 0);
        pf(
            &mut Lstep,
            x,
            y,
            0.5 * (step_size as f32) * (x_pos - x_neg + y_pos),
        );
    }
    {
        let x = 0;
        let x_pos = eval(c, Ld, x, y, 1, 0);
        let y_pos = eval(c, Ld, x, y, 0, -1);
        pf(&mut Lstep, x, y, 0.5 * (step_size as f32) * (x_pos + y_pos));
    }
    {
        let x = Lstep.width() - 1;
        let x_neg = eval(c, Ld, x, y, -1, 0);
        let y_pos = eval(c, Ld, x, y, 0, -1);
        pf(
            &mut Lstep,
            x,
            y,
            0.5 * (step_size as f32) * (-x_neg + y_pos),
        );
    }
    // First and last columns
    for y in 1..(Lstep.height() - 1) {
        {
            let x = 0;
            let x_pos = eval(c, Ld, x, y, 1, 0);
            let y_pos = eval(c, Ld, x, y, 0, 1);
            let y_neg = eval(c, Ld, x, y, 0, -1);
            pf(
                &mut Lstep,
                x,
                y,
                0.5 * (step_size as f32) * (-x_pos + y_pos - y_neg),
            );
        }
        {
            let x = Lstep.width() - 1;
            let y_pos = eval(c, Ld, x, y, 0, 1);
            let x_neg = eval(c, Ld, x, y, -1, 0);
            let y_neg = eval(c, Ld, x, y, 0, -1);
            pf(
                &mut Lstep,
                x,
                y,
                0.5 * (step_size as f32) * (-x_neg + y_pos - y_neg),
            );
        }
    }
    for x in 0..Lstep.width() {
        for y in 0..Lstep.height() {
            let Ld_pixel = gf(Ld, x, y);
            let Lstep_pixel = gf(Lstep, x, y);
            pf(&mut Lstep, x, y, Ld_pixel + Lstep_pixel);
        }
    }
}

/// Convenience method for calculating x_pos and x_neg that is more compact
#[allow(non_snake_case)]
pub fn eval(
    c: &GrayFloatImage,
    Ld: &GrayFloatImage,
    x: u32,
    y: u32,
    plus_x: i32,
    plus_y: i32,
) -> f32 {
    let x_set = (x as i32) + plus_x;
    let y_set = (y as i32) + plus_y;
    assert!(x_set >= 0);
    assert!(y_set >= 0);
    // If we access past the upper bounds of image the image class will assert
    (gf(c, x, y) + gf(c, x_set as u32, y)) * (gf(Ld, x_set as u32, y) - gf(Ld, x, y))
}
