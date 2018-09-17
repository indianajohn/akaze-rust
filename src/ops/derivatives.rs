use image;
use image::Pixel;
use types::image::GrayFloatImage;

pub fn sqrt_squared(image_1: &GrayFloatImage, image_2: &GrayFloatImage) -> GrayFloatImage {
    let mut result = GrayFloatImage::new(image_1.width(), image_1.height());
    assert!(image_1.width() == image_2.width());
    assert!(image_1.height() == image_2.height());
    for x in 0..image_1.width() {
        for y in 0..image_2.height() {
            let p1: f32 = image_1.get_pixel(x, y).channels()[0];
            let p2: f32 = image_2.get_pixel(x, y).channels()[0];
            let mut pixel = image_1.get_pixel(x, y).clone();
            pixel.channels_mut()[0] = f32::sqrt(p1 * p1 + p2 * p2);
            result.put_pixel(x, y, pixel);
        }
    }
    result
}

fn scharr_horizontal(image: &GrayFloatImage, scale_factor: f32) -> GrayFloatImage {
    let x = 3f32;
    let y = 10f32;
    let kernel: [f32; 9] = [
        -x / scale_factor, 0f32 / scale_factor, x / scale_factor, 
        -y / scale_factor, 0f32 / scale_factor, y / scale_factor,
        -x / scale_factor, 0f32 / scale_factor, x / scale_factor];
    image::imageops::filter3x3(image, &kernel)
}

fn scharr_vertical(image: &GrayFloatImage, scale_factor: f32) -> GrayFloatImage {
    let x = 3f32;
    let y = 10f32;
    let kernel: [f32; 9] = [
        -x / scale_factor, -y / scale_factor, -x / scale_factor,
        0f32 / scale_factor, 0f32 / scale_factor, 0f32 / scale_factor,
        x / scale_factor, y / scale_factor, x / scale_factor];
    image::imageops::filter3x3(image, &kernel)
}

pub fn scharr(image: &GrayFloatImage, x_order: bool, y_order: bool) -> GrayFloatImage {
    if x_order && y_order {
        let horizontal = sqrt_squared(&scharr_horizontal(&image, 1f32), &scharr_horizontal(&image, -1f32));
        let vertical = sqrt_squared(&scharr_vertical(&image, 1f32), &scharr_vertical(&image, -1f32));
        sqrt_squared(&vertical, &horizontal)
    } else if x_order {
        sqrt_squared(&scharr_horizontal(&image, 1f32), &scharr_horizontal(&image, -1f32))
    } else if y_order {
        sqrt_squared(&scharr_vertical(&image, 1f32), &scharr_vertical(&image, -1f32))
    } else {
        GrayFloatImage::new(image.width(), image.height())
    }
}
