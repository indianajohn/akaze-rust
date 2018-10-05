use image::DynamicImage;
use image::GenericImageView;
use image::GrayImage;
use image::Luma;
use image::Pixel;
use std::f32;
use std::path::PathBuf;

/// The image type we use in this library.
/// This is simply a wrapper around a contiguous vector. I would
/// typically err on the side of of avoiding premature optimization,
/// and using a higher-level interface for images. However, at first,
/// I tried just using the image crate's types with f32 as a
/// template type. All operations were approximately 40% slower.
///
/// The below traits have been violated in various parts of this crate,
/// with some image operations applying directly to the buffer. This,
/// again, ended up being a necessary optimization. Using iterators
/// to perform image filters sped them up in some cases by a factor of
/// 2.
#[derive(Debug, Clone)]
pub struct GrayFloatImage {
    pub buffer: Vec<f32>,
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
            buffer: vec![0f32; width * height],
            height: height,
            width: width,
        }
    }

    fn get(&self, x: usize, y: usize) -> f32 {
        self.buffer[self.width * y + x]
    }

    fn put(&mut self, x: usize, y: usize, pixel_value: f32) {
        self.buffer[self.width * y + x] = pixel_value;
    }
    fn half_size(&self) -> Self {
        let width = self.width() / 2;
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

/// Create a unit float image from the image crate'd DynamicImage type.
/// `input_image` - the input image.
/// # Return value
/// An image with pixel values between 0 and 1.
pub fn create_unit_float_image(input_image: &DynamicImage) -> GrayFloatImage {
    let gray_image: GrayImage = input_image.to_luma();
    let mut output_image =
        GrayFloatImage::new(input_image.width() as usize, input_image.height() as usize);
    for (x, y, gray_pixel) in gray_image.enumerate_pixels() {
        let pixel_value: u8 = gray_pixel.channels()[0];
        let scaled_float = (pixel_value as f32) * 1f32 / 255f32;
        output_image.put(x as usize, y as usize, scaled_float);
    }
    output_image
}

/// Generate a dynamic image from a GrayFloatImage
/// `input_image` - the input image.
/// # Return value
/// A dynamic image (can be written to file, etc..)
pub fn create_dynamic_image(input_image: &GrayFloatImage) -> DynamicImage {
    let mut output_image =
        DynamicImage::new_luma8(input_image.width() as u32, input_image.height() as u32);
    for x in 0..input_image.width() {
        for y in 0..input_image.height() {
            let pixel_value: f32 = input_image.get(x, y);
            let u8_pixel: u8 = (pixel_value * 255f32) as u8;
            output_image
                .as_mut_luma8()
                .unwrap()
                .put_pixel(x as u32, y as u32, Luma([u8_pixel]));
        }
    }
    output_image
}

/// Normalize an image between 0 and 1
/// `input_image` - the input image.
/// # Return value
/// the normalized image.
pub fn normalize(input_image: &GrayFloatImage) -> GrayFloatImage {
    let mut min_pixel = f32::MAX;
    let mut max_pixel = f32::MIN;
    let mut output_image =
        GrayFloatImage::new(input_image.width() as usize, input_image.height() as usize);
    for x in 0..input_image.width() {
        for y in 0..input_image.height() {
            let pixel = input_image.get(x, y);
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

/// Helper function to write an image to a file.
/// `input_image` - the image to write.
/// `path` - the path to which to write the image.
pub fn save(input_image: &GrayFloatImage, path: PathBuf) {
    if input_image.width() > 0 && input_image.height() > 0 {
        let normalized_image = normalize(&input_image);
        let dynamic_image = create_dynamic_image(&normalized_image);
        dynamic_image.save(path).unwrap();
    }
}

/// Return sqrt(image_1[i] + image_2[i]) for all pixels in the input images.
/// Save the result in image_1.
/// `image_1` - the first image.
/// `image_2` - the second image.
pub fn sqrt_squared(image_1: &mut GrayFloatImage, image_2: &GrayFloatImage) {
    debug_assert!(image_1.width() == image_2.width());
    debug_assert!(image_1.height() == image_2.height());
    let length = image_1.width() * image_1.height();
    let slice_1 = &mut image_1.buffer[..];
    let slice_2 = &image_2.buffer[..];
    let mut itr1 = slice_1.iter_mut();
    let mut itr2 = slice_2.iter();
    for _ in 0..(length) {
        let p1 = itr1.next().unwrap();
        let p2 = itr2.next().unwrap();
        *p1 += *p2;
    }
}

/// Fill border with neighboring pixels. A way of preventing instability
/// around the image borders for things like derivatives.
/// `output` - the image to operate upon.
/// `half_width` the number of pixels around the borders to operate on.
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

/// Horizontal image filter for variable kernel sizes.
/// `image` - the input image.
/// `kernel` the kernel to apply.
/// # Return value
/// The filter result.
#[inline(always)]
pub fn horizontal_filter(image: &GrayFloatImage, kernel: &Vec<f32>) -> GrayFloatImage {
    // Cannot have an even-sized kernel
    debug_assert!(kernel.len() % 2 == 1);
    let half_width = (kernel.len() / 2) as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut output = GrayFloatImage::new(image.width(), image.height());
    {
        let out_slice = &mut output.buffer[..];
        let image_slice = &image.buffer[..];
        for k in -half_width..=half_width {
            let mut out_itr = out_slice.iter_mut();
            let mut image_itr = image_slice.iter();
            let mut out_ptr = out_itr.nth(half_width as usize).unwrap();
            let mut image_val = image_itr.nth((half_width + k) as usize).unwrap();
            let kernel_value = kernel[(k + half_width) as usize];
            for _ in half_width..(w*h - half_width - 1) {
                *out_ptr += kernel_value * image_val;
                out_ptr = out_itr.next().unwrap();
                image_val = image_itr.next().unwrap();
            }
        }
    }
    fill_border(&mut output, half_width as usize);
    output
}

/// Vertical image filter for variable kernel sizes.
/// `image` - the input image.
/// `kernel` the kernel to apply.
/// # Return value
/// The filter result.
#[inline(always)]
pub fn vertical_filter(image: &GrayFloatImage, kernel: &Vec<f32>) -> GrayFloatImage {
    // Cannot have an even-sized kernel
    debug_assert!(kernel.len() % 2 == 1);
    let half_width = (kernel.len() / 2) as i32;
    let w = image.width() as i32;
    let h = image.height() as i32;
    let mut output = GrayFloatImage::new(image.width(), image.height());
    {
        let out_slice = &mut output.buffer[..];
        let image_slice = &image.buffer[..];
        for k in -half_width..=half_width {
            let mut out_itr = out_slice.iter_mut();
            let mut image_itr = image_slice.iter();
            let mut out_ptr = out_itr.nth((half_width *w ) as usize).unwrap();
            let mut image_val = image_itr.nth(((half_width*w) + (k*w) ) as usize).unwrap();
            let kernel_value = kernel[(k + half_width) as usize];
            for _ in (half_width * w)..(w*h - (half_width*w) - 1) {
                *out_ptr += kernel_value * image_val;
                out_ptr = out_itr.next().unwrap();
                image_val = image_itr.next().unwrap();
            }
        }
    }
    fill_border(&mut output, half_width as usize);
    output
}

/// The Gaussian function.
/// `x` the offset.
/// `r` sigma.
/// # Return value
/// The kernel value at x.
fn gaussian(x: f32, r: f32) -> f32 {
    ((2.0 * f32::consts::PI).sqrt() * r).recip() * (-x.powi(2) / (2.0 * r.powi(2))).exp()
}

/// Generate a Gaussina kernel.
/// `r` sigma.
/// `kernel_size` the size of the kernel.
/// # Return value
/// The kernel (a vector).
fn gaussian_kernel(r: f32, kernel_size: usize) -> Vec<f32> {
    let mut kernel = vec![0f32; kernel_size];
    let half_width = (kernel_size / 2) as i32;
    for i in -half_width..half_width {
        kernel[(i + half_width) as usize] = gaussian(i as f32, r);
    }
    kernel
}

/// Perform Gaussian blur on an image.
/// `r` sigma.
/// `kernel_size` the size of the kernel.
/// # Return value
/// The resulting image after the filter was applied.
pub fn gaussian_blur(image: &GrayFloatImage, r: f32, kernel_size: usize) -> GrayFloatImage {
    // a separable Gaussian kernel
    let kernel = gaussian_kernel(r, kernel_size);
    let img_horizontal = horizontal_filter(&image, &kernel);
    vertical_filter(&img_horizontal, &kernel)
}
