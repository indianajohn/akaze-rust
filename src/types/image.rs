use image::DynamicImage;
use image::GenericImageView;
use image::GrayImage;
use image::Luma;
use image::Pixel;
use std::f32;
use std::path::PathBuf;


// Using the image crate with u32 pixels is approximately 40% slower
// than this.
#[derive(Debug, Clone)]
pub struct GrayFloatImage {
    buffer: Vec<f32>,
    width: usize,
    height: usize,
}
pub trait ImageFunctions {
    /// The width of the image.
    /// # Return value
    /// The width.
    fn width(&self) -> usize;

    /// The height of the image.
    /// # Return value
    /// The height.
    fn height(&self) -> usize;

    /// Create a new image
    /// `width` width of image
    /// `height` height of image.
    /// # Return value
    /// The image.
    fn new(width: usize, height: usize) -> Self;

    /// Return an image with each dimension halved
    fn half_size(&self) -> Self;

    /// get a float pixel at x, y
    /// `x` x coordinate.
    /// `y` y coordinate.
    /// # Return value
    /// the value of the pixel.
    fn get(&self, x: usize, y: usize) -> f32;

    /// put a float pixel to x, y
    /// `x` x coordinate.
    /// `y` y coordinate.
    /// pixel_value: value to put
    fn put(&mut self, x: usize, y: usize, pixel_value: f32);
}

impl ImageFunctions for GrayFloatImage {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn new(width: usize, height: usize) -> Self {
        Self { 
            buffer: vec![0f32; width * height], height: height, width: width 
            }
    }

    fn get(&self, x: usize, y: usize) -> f32 {
        self.buffer[self.width * y + x]
    }

    fn put(&mut self, x: usize, y: usize, pixel_value: f32) {
        self.buffer[self.width * y + x] = pixel_value;
    }
    fn half_size(&self) -> Self {
        let width  = self.width() / 2;
        let height = self.height() / 2;
        let mut out = Self::new(width, height);
        for x in 0..width {
            for y in 0..height {
                let mut val = 0f32;
                for x_src in (2 * x)..(2 * x + 2) {
                    for y_src in (2 * y)..(2 * y + 2) {
                        val += self.get(x_src, y_src);
                    }
                }
                out.put(x, y, val / 4f32);
            }
        }
        out
    }
}

pub fn create_unit_float_image(input_image: &DynamicImage) -> GrayFloatImage {
    let gray_image: GrayImage = input_image.to_luma();
    let mut output_image = GrayFloatImage::new(
        input_image.width() as usize, input_image.height() as usize);
    for (x, y, gray_pixel) in gray_image.enumerate_pixels() {
        let pixel_value: u8 = gray_pixel.channels()[0];
        let scaled_float = (pixel_value as f32) * 1f32 / 255f32;
        output_image.put(x as usize, y as usize, scaled_float);
    }
    output_image
}

pub fn create_dynamic_image(input_image: &GrayFloatImage) -> DynamicImage {
    let mut output_image = DynamicImage::new_luma8(input_image.width() as u32, input_image.height() as u32);
    for x in 0..input_image.width() {
        for y in 0..input_image.height() {
            let pixel_value: f32 = input_image.get(x,y);
            let u8_pixel: u8 = (pixel_value * 255f32) as u8;
            output_image
                .as_mut_luma8()
                .unwrap()
                .put_pixel(x as u32, y as u32, Luma([u8_pixel]));
        }
    }
    output_image
}

pub fn normalize(input_image: &GrayFloatImage) -> GrayFloatImage {
    let mut min_pixel = f32::MAX;
    let mut max_pixel = f32::MIN;
    let mut output_image = GrayFloatImage::new(
        input_image.width() as usize, input_image.height() as usize);
    for x in 0..input_image.width() {
        for y in 0..input_image.height() {
            let pixel = input_image.get(x,y);
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
            let mut pixel = input_image.get(x, y);
            pixel = pixel - min_pixel;
            pixel = pixel / new_max_pixel;
            output_image.put(x, y, pixel);
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

pub fn sqrt_squared(image_1: &GrayFloatImage, image_2: &GrayFloatImage) -> GrayFloatImage {
    let mut result = GrayFloatImage::new(image_1.width(), image_1.height());
    debug_assert!(image_1.width() == image_2.width());
    debug_assert!(image_1.height() == image_2.height());
    for x in 0..image_1.width() {
        for y in 0..image_2.height() {
            let p1: f32 = image_1.get(x, y);
            let p2: f32 = image_2.get(x, y);
            result.put(x, y, f32::sqrt(p1 * p1 + p2 * p2));
        }
    }
    result
}

pub fn fill_border(output: &mut GrayFloatImage, half_width: usize) {
    for x in 0..output.width() {
        let plus = output.get(x, half_width);
        let minus = output.get(x, output.height() - half_width - 1);
        for y in 0..half_width {
            output.put(x, y, plus);
        }
        for y in (output.height() - half_width)..output.height() {
            output.put(x, y, minus);
        }
    }
    for y in 0..output.height() {
        let plus = output.get(half_width, y);
        let minus = output.get(output.width() - half_width - 1, y);
        for x in 0..half_width {
            output.put(x, y, plus);
        }
        for x in (output.width() - half_width)..output.width() {
            output.put(x, y, minus);
        }
    }
}

pub fn horizontal_filter(image: &GrayFloatImage, kernel: &Vec<f32>) -> GrayFloatImage {
    // Cannot have an even-sized kernel
    debug_assert!(kernel.len() % 2 == 1);
    let half_width = (kernel.len() / 2) as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut output = GrayFloatImage::new(image.width(), image.height());
    // center of image
    for y in 0..h {
        let start_i = (w * y) as usize;
        let stop_i = (w * (y + 1)) as usize;
        let image_slice = &image.buffer[start_i..stop_i];
        let out_slice = &mut output.buffer[start_i..stop_i];
        for x in half_width..(w - half_width) {
            let mut val = 0f32;
            for k in -half_width..=half_width {
                let i = k + half_width;
                let new_x = (x + k) as usize;
                val += kernel[i as usize] * image_slice[new_x];
            }
            out_slice[x as usize] = val;
        }
    }
    fill_border(&mut output, half_width as usize);
    output
}

pub fn vertical_filter(image: &GrayFloatImage, kernel: &Vec<f32>) -> GrayFloatImage {
    // Cannot have an even-sized kernel
    debug_assert!(kernel.len() % 2 == 1);
    let half_width = (kernel.len() / 2) as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut output = GrayFloatImage::new(image.width(), image.height());
    // center of image
    for y in half_width..(h - half_width) {
        let start_i = (w * y) as usize;
        let stop_i = (w * (y + 1)) as usize;
        let out_slice = &mut output.buffer[start_i..stop_i];
        for x in 0..w {
            let mut val = 0f32;
            for k in -half_width..=half_width {
                let i = k + half_width;
                let new_y = y + k;
                val += kernel[i as usize] * image.get(x as usize, new_y as usize);
            }
            out_slice[x as usize] = val;
        }
    }
    fill_border(&mut output, half_width as usize);
    output
}

fn gaussian(x: f32, r: f32) -> f32 {
    ((2.0 * f32::consts::PI).sqrt() * r).recip() * (-x.powi(2) / (2.0 * r.powi(2))).exp()
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
