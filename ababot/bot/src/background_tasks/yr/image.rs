use std::error::Error;

use image::{RgbImage, imageops::brighten};

use super::types::Series;

pub fn create_image(data: Vec<Series>) -> Result<RgbImage, Box<dyn Error>> {
    let imgx = 1000;
    let imgy = 1000;

    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.5 * x as f32) as u8;
        let b = (0.5 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);
    }

    imgbuf = brighten(&imgbuf, 100);

    for x in (imgx / 10..imgx).step_by((imgx / 10).try_into().unwrap()) {
        for y in imgy / 2..imgy {
            let pixel = imgbuf.get_pixel_mut(x, y);
            *pixel = image::Rgb([255, 255, 255]);
        }
    }

    Ok(RgbImage::new(1, 1))
}
