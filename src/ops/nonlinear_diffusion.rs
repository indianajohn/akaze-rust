use types::evolution::EvolutionStep;
use types::image::{GrayFloatImage, ImageFunctions};
//use std::io;
/// This function performs a scalar non-linear diffusion step
/// `Ld` Output image in the evolution
/// `c` Conductivity image. The function c is a scalar value that depends on the gradient norm
/// `Lstep` Previous image in the evolution
/// `step_size` The step size in time units
/// Forward Euler Scheme 3x3 stencil
/// dL_by_ds = d(c dL_by_dx)_by_dx + d(c dL_by_dy)_by_dy
#[allow(non_snake_case)]
pub fn calculate_step(evolution_step: &mut EvolutionStep, step_size: f64) {
    let Ld: &mut GrayFloatImage = &mut evolution_step.Lt;
    let c: &GrayFloatImage = &evolution_step.Lflow;
    let Lstep: &mut GrayFloatImage = &mut evolution_step.Lstep;

    // Diffusion all the image except borders
    for y in 1..(Lstep.height() - 1) {
        for x in 1..(Lstep.width() - 1) {
            let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
            let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
            let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
            let y_neg = eval(c, Ld, x, y, 0, 0, 0, 0, -1, 0, 0, -1);
            Lstep.put(
                x,
                y,
                0.5 * (step_size as f32) * (x_pos - x_neg + y_pos - y_neg),
            );
        }
    }
    // First row
    for x in 1..(Lstep.width() - 1) {
        let y = 0;
        let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
        let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
        Lstep.put(
            x,
            y,
            0.5 * (step_size as f32) * (x_pos - x_neg + y_pos),
        );
    }
    {
        let x = 0;
        let y = 0;
        let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
        Lstep.put(x, y, 0.5 * (step_size as f32) * (x_pos + y_pos));
    }
    {
        let x = Lstep.width() - 1;
        let y = 0;
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
        let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
        Lstep.put(
            x,
            y,
            0.5 * (step_size as f32) * (-x_neg + y_pos),
        );
    }
    // Last row
    let y = Lstep.height() - 1;
    for x in 1..(Lstep.width() - 1) {
        let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, -1, -1, 0);
        let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
        Lstep.put(
            x,
            y,
            0.5 * (step_size as f32) * (x_pos - x_neg + y_pos),
        );
    }
    {
        let x = 0;
        let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, -1, -1, 0);
        Lstep.put(x, y, 0.5 * (step_size as f32) * (x_pos + y_pos));
    }
    {
        let x = Lstep.width() - 1;
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, -1, -1, 0);
        let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
        Lstep.put(
            x,
            y,
            0.5 * (step_size as f32) * (-x_neg + y_pos),
        );
    }
    // First and last columns
    for y in 1..(Lstep.height() - 1) {
        {
            let x = 0;
            let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
            let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
            let y_neg = eval(c, Ld, x, y, 0, 0, 0, 0, -1, 0, 0, -1);
            Lstep.put(
                x,
                y,
                0.5 * (step_size as f32) * (x_pos + y_pos - y_neg),
            );
        }
        {
            let x = Lstep.width() - 1;
            let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
            let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
            let y_neg = eval(c, Ld, x, y, 0, 0, 0, 0, -1, 0, 0, -1);
            Lstep.put(
                x,
                y,
                0.5 * (step_size as f32) * (-x_neg + y_pos - y_neg),
            );
        }
    }
    for x in 0..Lstep.width() {
        for y in 0..Lstep.height() {
            let Ld_pixel = Ld.get(x, y);
            let Lstep_pixel = Lstep.get(x, y);
            Ld.put(x, y, Ld_pixel + Lstep_pixel);
        }
    }
}

/// Convenience method for calculating x_pos and x_neg that is more compact
#[allow(non_snake_case)]
pub fn eval(
    c: &GrayFloatImage,
    Ld: &GrayFloatImage,
    x: usize,
    y: usize,
    plus_x_1: i32,
    plus_x_2: i32,
    plus_x_3: i32,
    plus_x_4: i32,
    plus_y_1: i32,
    plus_y_2: i32,
    plus_y_3: i32,
    plus_y_4: i32,
) -> f32 {
    let set_x_1 = (x as i32) + plus_x_1;
    debug_assert!(set_x_1 >= 0);
    let set_x_2 = (x as i32) + plus_x_2;
    debug_assert!(set_x_2 >= 0);
    let set_x_3 = (x as i32) + plus_x_3;
    debug_assert!(set_x_3 >= 0);
    let set_x_4 = (x as i32) + plus_x_4;
    debug_assert!(set_x_4 >= 0);
    let set_y_1 = (y as i32) + plus_y_1;
    debug_assert!(set_y_1 >= 0);
    let set_y_2 = (y as i32) + plus_y_2;
    debug_assert!(set_y_2 >= 0);
    let set_y_3 = (y as i32) + plus_y_3;
    debug_assert!(set_y_3 >= 0);
    let set_y_4 = (y as i32) + plus_y_4;
    debug_assert!(set_y_4 >= 0);
    // If we access past the upper bounds of image the image class will assert
    (c.get(set_x_1 as usize, set_y_1 as usize) + c.get(set_x_2 as usize, set_y_2 as usize))
        * (Ld.get(set_x_3 as usize, set_y_3 as usize) - Ld.get(set_x_4 as usize, set_y_4 as usize))
}
