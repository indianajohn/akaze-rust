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

fn scharr_horizontal(image: &GrayFloatImage) -> GrayFloatImage {
    let kernel: [f32; 9] = [-3f32, 0f32, 3f32, -10f32, 0f32, 10f32, -3f32, 0f32, 3f32];
    image::imageops::filter3x3(image, &kernel)
}

fn scharr_vertical(image: &GrayFloatImage) -> GrayFloatImage {
    let kernel: [f32; 9] = [-3f32, -10f32, -3f32, 0f32, 0f32, 0f32, 3f32, 10f32, 3f32];
    image::imageops::filter3x3(image, &kernel)
}

pub fn scharr(image: &GrayFloatImage, x_order: bool, y_order: bool) -> GrayFloatImage {
    if x_order && y_order {
        sqrt_squared(&scharr_vertical(&image), &scharr_horizontal(&image))
    } else if x_order {
        scharr_horizontal(image)
    } else if y_order {
        scharr_vertical(image)
    } else {
        GrayFloatImage::new(image.width(), image.height())
    }
}
