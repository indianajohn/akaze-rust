use image::DynamicImage;
use image::GenericImage;
use image::GrayImage;
use image::ImageBuffer;
use image::Luma;
use image::Pixel;
use std::path::PathBuf;
use std::f32;
pub type GrayFloatImage = ImageBuffer<Luma<f32>, Vec<f32>>;

pub fn create_unit_float_image(input_image: &DynamicImage) -> GrayFloatImage {
    let gray_image: GrayImage = input_image.to_luma();
    let mut output_image = GrayFloatImage::new(input_image.width(), input_image.height());
    for (x, y, gray_pixel) in gray_image.enumerate_pixels() {
        let pixel_value: u8 = gray_pixel.channels()[0];
        let scaled_float = (pixel_value as f32) * 1f32 / 255f32;
        output_image.put_pixel(x, y, Luma([scaled_float]));
    }
    output_image
}

pub fn create_dynamic_image(input_image: &GrayFloatImage) -> DynamicImage {
    let mut output_image = DynamicImage::new_luma8(input_image.width(), input_image.height());
    for (x, y, float_pixel) in input_image.enumerate_pixels() {
        let pixel_value: f32 = float_pixel.channels()[0];
        let u8_pixel: u8 = (pixel_value * 255f32) as u8;
        output_image
            .as_mut_luma8()
            .unwrap()
            .put_pixel(x, y, Luma([u8_pixel]));
    }
    output_image
}

pub fn normalize(input_image: &GrayFloatImage
) -> GrayFloatImage {
    let mut min_pixel = f32::MAX;
    let mut max_pixel = f32::MIN;
    let mut output_image = GrayFloatImage::new(input_image.width(), input_image.height());
    for x in 0..input_image.width() {
        for y in 0..input_image.height() {
            let pixel = gf(&input_image, x, y);
            if pixel > max_pixel {
                max_pixel = pixel;
            }
            if pixel < min_pixel {
                min_pixel = pixel;
            }
        }
    }

    let new_max_pixel = max_pixel - min_pixel;
    for x in 0..input_image.width() {
        for y in 0..input_image.height() {
            let mut pixel = gf(&input_image, x, y);
            pixel = pixel - min_pixel;
            pixel = pixel / new_max_pixel;
            pf(&mut output_image, x, y, pixel);
        }
    }
    output_image
}

pub fn save(input_image: &GrayFloatImage, path: PathBuf) {
    if input_image.width() > 0 && input_image.height() > 0 {
        let normalized_image = normalize(&input_image);
        let dynamic_image = create_dynamic_image(&normalized_image);
        dynamic_image.save(path).unwrap();
    }
}

/// get a float pixel at x, y
/// `x` x coordinate.
/// `y` y coordinate.
/// # Return value
/// the value of the pixel.
pub fn gf(image: &GrayFloatImage, x: u32, y: u32) -> f32 {
    image.get_pixel(x, y).channels()[0]
}

/// put a float pixel to x, y
/// `x` x coordinate.
/// `y` y coordinate.
pub fn pf(image: &mut GrayFloatImage, x: u32, y: u32, pixel_value: f32) {
    image.put_pixel(x, y, Luma([pixel_value]));
}

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

pub fn fill_border(mut output: &mut GrayFloatImage) {
    // first and last row - copy nearest pixel
    for x in 0..output.width() {
        let plus = gf(&output, x, 1);
        let minus = gf(&output, x, output.height() - 2);
        pf(&mut output, x, 0, plus);
        let y = output.height() - 1;
        pf(&mut output, x, y, minus);
    }
    // first and last column - copy nearest pixel
    for y in 0..output.height() {
        let plus = gf(&output, 1, y);
        let minus = gf(&output, output.width() - 2, y);
        pf(&mut output, 0, y, plus);
        let x = output.width() - 1;
        pf(&mut output, x, y, minus);
    }
}

pub fn horizontal_filter(image: &GrayFloatImage, kernel: &[f32; 3] ) -> GrayFloatImage {
    assert!(kernel.len() == 3);
    let mut output = GrayFloatImage::new(image.width(), image.height());
    // center of image
    for x in 1..(image.width() - 1) {
        for y in 1..(image.height() - 1) {
            let val = kernel[0] * gf(image,x - 1, y) + kernel[1] * gf(image, x, y) + kernel[2] * gf(image, x + 1, y);
            pf(&mut output, x, y, val);
        }
    }
    fill_border(&mut output);
    output
}

pub fn vertical_filter(image: &GrayFloatImage, kernel: &[f32; 3] ) -> GrayFloatImage {
    assert!(kernel.len() == 3);
    let mut output = GrayFloatImage::new(image.width(), image.height());
    // center of image
    for x in 1..(image.width() - 1) {
        for y in 1..(image.height() - 1) {
            let val = kernel[0] * gf(image,x, y - 1) + kernel[1] * gf(image, x, y) + kernel[2] * gf(image, x, y + 1);
            pf(&mut output, x, y, val);
        }
    }
    fill_border(&mut output);
    output
}