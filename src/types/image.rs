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
