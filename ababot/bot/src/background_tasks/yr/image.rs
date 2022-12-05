use std::error::Error;

use chrono::{DateTime, Local, Timelike};
use chrono_tz::Europe::Oslo;
use chrono_tz::Tz;
use image::{
    imageops::{flip_vertical_in_place, overlay},
    RgbaImage,
};
use text_to_png::{Color, TextRenderer};

use super::types::Series;

const IMGX: u32 = 1000;
const IMGY: u32 = 1000;

pub fn create_image(data: Vec<Series>) -> Result<RgbaImage, Box<dyn Error>> {
    let mut imgbuf = image::ImageBuffer::new(IMGX, IMGY);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgba([r, 0, b, 255]);
    }

    for x in (IMGX / 10..IMGX).step_by((IMGX / 10).try_into().unwrap()) {
        for y in IMGY / 2..IMGY {
            let pixel = imgbuf.get_pixel_mut(x, y);
            *pixel = image::Rgba([255, 255, 255, 255]);
        }
    }

    let hour_now = Local::now().hour();

    let weather_series_filtered = data
        .into_iter()
        .filter(|w| {
            let hour = DateTime::parse_from_rfc3339(&w.time)
                .map_err(|e| e.to_string())
                .unwrap()
                .hour();
            hour >= hour_now && hour < hour_now + 10
        })
        .collect::<Vec<_>>();

    // Get first 10 hours
    let weather_series = weather_series_filtered.chunks(10).next().unwrap();

    let hours: Vec<f32> = weather_series
        .iter()
        .clone()
        .map(|w| {
            w.data
                .next_1_hours
                .as_ref()
                .unwrap()
                .details
                .precipitation_amount as f32
        })
        .collect();
    let rain_image = add_rain(hours);
    overlay(&mut imgbuf, &rain_image, 0, 0);

    let hours: Vec<DateTime<Tz>> = weather_series
        .iter()
        .map(|w| {
            DateTime::parse_from_rfc3339(&w.time)
                .map_err(|e| e.to_string())
                .unwrap()
                .with_timezone(&Oslo)
        })
        .collect();
    let degrees = weather_series
        .iter()
        .map(|w| w.data.instant.details.air_temperature as f32)
        .collect::<Vec<_>>();
    let time_image = add_times_and_temperature(hours, degrees);
    overlay(&mut imgbuf, &time_image, 0, 0);

    let current_date = Local::now().format("%d.%m").to_string();
    let renderer = TextRenderer::default();
    let date_image =
        renderer.render_text_to_png_data(current_date, 128, Color::new(255, 255, 255))?;
    let date_image = image::load_from_memory(&date_image.data)?;
    overlay(&mut imgbuf, &date_image, 100, 100);

    // TODO: Make this work
    // let path =  include_bytes!("resources/wi-cloud.png");
    // let mut icon = image::load_from_memory(path).unwrap().into_rgba8();
    // icon = resize(&icon, 200, 200, imageops::FilterType::Lanczos3);
    // overlay(&mut imgbuf, &icon, 700, 100);

    Ok(imgbuf)
}

fn add_rain(rain: Vec<f32>) -> RgbaImage {
    if rain.len() != 10 {
        panic!("rain must be 10 elements long");
    }

    let scalar = rain.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

    let scaled_rain = rain
        .iter()
        .map(|x| x * 500f32 / scalar)
        .collect::<Vec<f32>>();

    let mut imgbuf = RgbaImage::new(IMGX, IMGY);

    for step in (0u32..1000).step_by((IMGX / 10).try_into().unwrap()) {
        for x in step + 1..step + 100 {
            for y in 0..scaled_rain[(x / 100) as usize].round() as u32 {
                let pixel = imgbuf.get_pixel_mut(x, y);
                *pixel = image::Rgba([25, 82, 242, 255]);
            }
        }
    }

    flip_vertical_in_place(&mut imgbuf);

    let renderer = TextRenderer::default();
    let unit_image = renderer
        .render_text_to_png_data(format!("{}mm", scalar), 32, Color::new(255, 255, 255))
        .unwrap();
    let unit_image = image::load_from_memory(&unit_image.data).unwrap();
    let unit_image_width = unit_image.width() as i64;
    image::imageops::overlay(&mut imgbuf, &unit_image, 1000 - unit_image_width, 500);

    let unit_image_2 = renderer
        .render_text_to_png_data(
            format!("{}mm", scalar / 2f32),
            32,
            Color::new(255, 255, 255),
        )
        .unwrap();
    let unit_image_2 = image::load_from_memory(&unit_image_2.data).unwrap();
    let unit_image_2_width = unit_image_2.width() as i64;
    image::imageops::overlay(&mut imgbuf, &unit_image_2, 1000 - unit_image_2_width, 750);

    imgbuf
}

fn add_times_and_temperature(times: Vec<DateTime<Tz>>, degrees: Vec<f32>) -> RgbaImage {
    if times.len() != 10 {
        panic!("times must be 10 elements long");
    }

    let mut imgbuf = RgbaImage::new(IMGX, IMGY);

    let renderer = TextRenderer::default();
    for i in 0i64..10 {
        let time = times[i as usize];
        let time_image = renderer
            .render_text_to_png_data(time.format("%H").to_string(), 32, Color::new(255, 255, 255))
            .unwrap();
        let time_image = image::load_from_memory(&time_image.data).unwrap();
        image::imageops::overlay(&mut imgbuf, &time_image, 100 * i + 40, 400);
        let degree = degrees[i as usize];
        let degree_image = renderer
            .render_text_to_png_data(format!("{}°", degree), 32, Color::new(255, 255, 255))
            .unwrap();
        let degree_image = image::load_from_memory(&degree_image.data).unwrap();
        image::imageops::overlay(&mut imgbuf, &degree_image, 100 * i + 30, 450);
    }
    imgbuf
}
