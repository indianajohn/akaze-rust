use types::image::{horizontal_filter, sqrt_squared, vertical_filter, GrayFloatImage, ImageFunctions};

fn scharr_horizontal(image: &GrayFloatImage) -> GrayFloatImage {
    // a separable Scharr kernel
    let k_horizontal = vec![3f32, 10f32, 3f32];
    let k_vertical = vec![-1f32, 0f32, 1f32];
    let img_horizontal = horizontal_filter(&image, &k_horizontal);
    vertical_filter(&img_horizontal, &k_vertical)
}

fn scharr_vertical(image: &GrayFloatImage) -> GrayFloatImage {
    // a separable Scharr kernel
    let k_vertical = vec![3f32, 10f32, 3f32];
    let k_horizontal = vec![-1f32, 0f32, 1f32];
    let img_horizontal = horizontal_filter(&image, &k_horizontal);
    vertical_filter(&img_horizontal, &k_vertical)
}

pub fn scharr(image: &GrayFloatImage, x_order: bool, y_order: bool) -> GrayFloatImage {
    if x_order && y_order {
        let horizontal = scharr_horizontal(&image);
        let mut vertical = scharr_horizontal(&image);
        sqrt_squared(&mut vertical, &horizontal);
        vertical
    } else if x_order {
        scharr_horizontal(&image)
    } else if y_order {
        scharr_vertical(&image)
    } else {
        GrayFloatImage::new(image.width(), image.height())
    }
}

pub fn scharr_variable_kernel(
    image: &GrayFloatImage, x_order: bool, y_order: bool, sigma_size: u32) -> GrayFloatImage {
    // TODO
    GrayFloatImage::new(image.width(), image.height())
}