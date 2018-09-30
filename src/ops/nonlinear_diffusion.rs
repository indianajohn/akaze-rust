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
    let w = Lstep.width();
    let h = Lstep.height();

    // Diffusion all the image except borders
    for y in 1..(h - 1) {
        let mut Ld_yn = Ld.buffer.iter();
        let mut Ld_yn_i = Ld_yn.nth(w * (y - 1) + 1).unwrap();

        let mut Ld_yp = Ld.buffer.iter();
        let mut Ld_yp_i = Ld_yp.nth(w * (y + 1) + 1).unwrap();

        let mut Ld_xn = Ld.buffer.iter();
        let mut Ld_xn_i = Ld_xn.nth(w * y + 0).unwrap();

        let mut Ld_x = Ld.buffer.iter();
        let mut Ld_x_i = Ld_x.nth(w * y + 1).unwrap();

        let mut Ld_xp = Ld.buffer.iter();
        let mut Ld_xp_i = Ld_xp.nth(w * y + 2).unwrap();

        let mut c_yn = c.buffer.iter();
        let mut c_yn_i = c_yn.nth(w * (y - 1) + 1).unwrap();

        let mut c_yp = c.buffer.iter();
        let mut c_yp_i = c_yp.nth(w * (y + 1) + 1).unwrap();

        let mut c_xn = c.buffer.iter();
        let mut c_xn_i = c_xn.nth(w * y + 0).unwrap();

        let mut c_x = c.buffer.iter();
        let mut c_x_i = c_x.nth(w * y + 1).unwrap();

        let mut c_xp = c.buffer.iter();
        let mut c_xp_i = c_xp.nth(w * y + 2).unwrap();

        let slice = &mut Lstep.buffer[(w * y + 1)..(w * y + w - 1)];
        for Lstep_x_i in slice.iter_mut() {
            let x_pos = (c_x_i + c_xp_i) * (Ld_xp_i - Ld_x_i);
            let x_neg = (c_xn_i + c_x_i) * (Ld_x_i - Ld_xn_i);
            let y_pos = (c_x_i + c_yp_i) * (Ld_yp_i - Ld_x_i);
            let y_neg = (c_yn_i + c_x_i) * (Ld_x_i - Ld_yn_i);
            *Lstep_x_i = 0.5 * (step_size as f32) * (x_pos - x_neg + y_pos - y_neg);

            c_x_i = c_x.next().unwrap();
            c_xp_i = c_xp.next().unwrap();
            c_xn_i = c_xn.next().unwrap();
            c_yp_i = c_yp.next().unwrap();
            c_yn_i = c_yn.next().unwrap();

            Ld_x_i = Ld_x.next().unwrap();
            Ld_xp_i = Ld_xp.next().unwrap();
            Ld_xn_i = Ld_xn.next().unwrap();
            Ld_yp_i = Ld_yp.next().unwrap();
            Ld_yn_i = Ld_yn.next().unwrap();
        }
    }
    // First row
    for x in 1..(Lstep.width() - 1) {
        let y = 0;
        let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
        let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
        Lstep.put(x, y, 0.5 * (step_size as f32) * (x_pos - x_neg + y_pos));
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
        Lstep.put(x, y, 0.5 * (step_size as f32) * (-x_neg + y_pos));
    }
    // Last row
    let y = Lstep.height() - 1;
    for x in 1..(Lstep.width() - 1) {
        let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
        let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, -1, -1, 0);
        let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
        Lstep.put(x, y, 0.5 * (step_size as f32) * (x_pos - x_neg + y_pos));
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
        Lstep.put(x, y, 0.5 * (step_size as f32) * (-x_neg + y_pos));
    }
    // First and last columns
    for y in 1..(Lstep.height() - 1) {
        {
            let x = 0;
            let x_pos = eval(c, Ld, x, y, 0, 1, 1, 0, 0, 0, 0, 0);
            let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
            let y_neg = eval(c, Ld, x, y, 0, 0, 0, 0, -1, 0, 0, -1);
            Lstep.put(x, y, 0.5 * (step_size as f32) * (x_pos + y_pos - y_neg));
        }
        {
            let x = Lstep.width() - 1;
            let y_pos = eval(c, Ld, x, y, 0, 0, 0, 0, 0, 1, 1, 0);
            let x_neg = eval(c, Ld, x, y, -1, 0, 0, -1, 0, 0, 0, 0);
            let y_neg = eval(c, Ld, x, y, 0, 0, 0, 0, -1, 0, 0, -1);
            Lstep.put(x, y, 0.5 * (step_size as f32) * (-x_neg + y_pos - y_neg));
        }
    }

    let mut Lstep_iter = Lstep.buffer.iter();
    for Ld_iter in Ld.buffer.iter_mut() {
        *Ld_iter += Lstep_iter.next().unwrap();
    }
}

/// Convenience method for calculating x_pos and x_neg that is more compact
#[allow(non_snake_case)]
#[inline(always)]
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
