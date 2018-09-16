use image::DynamicImage;
use image::GenericImage;
use image::GrayImage;
use image::ImageBuffer;
use image::Luma;
use image::Pixel;
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

/// get a float pixel at x, y
/// `x` x coordinate.
/// `y` y coordinate.
/// # Return value
/// the value of the pixel.
pub fn gf(image: &GrayFloatImage, x: u32, y: u32) -> f32 {
    image.get_pixel(x, y).channels()[0]
}
