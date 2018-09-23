use image::DynamicImage;
use image::GenericImage;
use image::GrayImage;
use image::ImageBuffer;
use image::Luma;
use image::Pixel;
use image::GenericImageView;
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

pub fn fill_border(mut output: &mut GrayFloatImage, half_width: u32) {
    for x in 0..output.width() {
        let plus = gf(&output, x, half_width);
        let minus = gf(&output, x, output.height() - half_width - 1);
        for y  in 0..half_width {
            pf(&mut output, x, y, plus);
        }
        for y  in (output.height() - half_width)..output.height() {
            pf(&mut output, x, y, minus);
        }
    }
    for y in 0..output.height() {
        let plus = gf(&output, half_width, y);
        let minus = gf(&output, output.width()  - half_width- 1, y);
        for x in 0..half_width {
            pf(&mut output, x, y, plus);
        }
        for x in (output.width() - half_width )..output.width() {
            pf(&mut output, x, y, minus);
        }
    }
}

pub fn horizontal_filter(image: &GrayFloatImage, kernel: &Vec<f32> ) -> GrayFloatImage {
    // Cannot have an even-sized kernel
    assert!(kernel.len() % 2 == 1);
    let half_width = (kernel.len() / 2) as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut output = GrayFloatImage::new(image.width(), image.height());
    // center of image
    for x in half_width..(w - half_width) {
        for y in 0..h {
            let mut val = 0f32;
            for k in -half_width..=half_width {
                let i = k + half_width;
                let new_x = x + k;
                val += kernel[i as usize] * gf(image, new_x as u32, y as u32);
            }
            pf(&mut output, x as u32, y as u32, val);
        }
    }
    fill_border(&mut output, half_width as u32);
    output
}

pub fn vertical_filter(image: &GrayFloatImage, kernel: &Vec<f32> ) -> GrayFloatImage {
    // Cannot have an even-sized kernel
    assert!(kernel.len() % 2 == 1);
    let half_width = (kernel.len() / 2) as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut output = GrayFloatImage::new(image.width(), image.height());
    // center of image
    for x in 0..w {
        for y in half_width..(h - half_width) {
            let mut val = 0f32;
            for k in -half_width..=half_width {
                let i = k + half_width;
                let new_y = y + k;
                val += kernel[i as usize] * gf(image, x as u32, new_y as u32);
            }
            pf(&mut output, x as u32, y as u32, val);
        }
    }
    fill_border(&mut output, half_width as u32);
    output
}

fn gaussian(x: f32, r: f32) -> f32 {
    ((2.0 * f32::consts::PI).sqrt() * r).recip() *
    (-x.powi(2) / (2.0 * r.powi(2))).exp()
}

fn gaussian_kernel(r: f32, kernel_size: usize) -> Vec<f32> {
    let mut kernel = vec![0f32; kernel_size];
    let half_width = (kernel_size / 2) as i32;
    for i in -half_width..half_width {
        kernel[(i + half_width) as usize] = gaussian(i as f32, r);
    }
    kernel
}
pub fn gaussian_blur(image: &GrayFloatImage, r: f32, kernel_size: usize) -> GrayFloatImage {
    // a separable Gaussian kernel
    let kernel = gaussian_kernel(r, kernel_size);
    let img_horizontal = horizontal_filter(&image, &kernel);
    vertical_filter(&img_horizontal, &kernel)
}
