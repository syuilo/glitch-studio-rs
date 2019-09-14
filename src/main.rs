extern crate image;
extern crate rand;

use image::Rgba;
use image::Rgb;
use std::io;
use std::cmp;
use image::GenericImage;
use image::GenericImageView;
use image::Pixel;
use rand::{Rng, SeedableRng};

struct GSDataModel {
    fxs: Vec<FX>,
}

struct FX {
    name: String,
}

const SEED: u64 = 0;
const SCALE: f32 = 1.0;
const GHOST_AMOUNT: u32 = 64;
const CHANNEL_SHIFT_AMOUNT: u32 = 8;
const TEAR_AMOUNT: u32 = 32;
const TEAR_MAX_HEIGHT: u32 = 64;
const TEAR_MAX_TIMES: u32 = 32;
const PIXEL_BLUR_AMOUNT: u32 = 256;
const PIXEL_BLUR_FLUCTUATION: u32 = 32;

macro_rules! cast_array {
    ($array:expr, $castTo:ty, $size:expr) => {{
        let mut arr: [$castTo; $size] = [0; $size];
        for (to, from) in arr.iter_mut().zip($array.iter()) {
           *to = *from as $castTo;
        }
        arr
    }}
}

fn main() {
    let data = GSDataModel {
        fxs: vec!(FX {
            name: "shift".to_string()
        })
    };

    println!("GLITCH STUDIO");

    let mut rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(SEED);

    println!("Please input your image path:");
/*
    let mut path = String::new();

    io::stdin().read_line(&mut path)
        .expect("Failed to read line");

    let path = path.trim();
*/
    let path = "input5.png";

    println!("Loading the image...");

    let img = image::open(path)
        .expect("Failed to load image");

    let (width, height) = img.dimensions();

    println!("Image {}x{}", width, height);

    let mut buffer = image::ImageBuffer::new(width, height);

    // Copy
    for x in 0..width {
        for y in 0..height {
            buffer.put_pixel(x, y, img.get_pixel(x, y));
        }
    }

    let img = buffer;

    let img = ghost(img, GHOST_AMOUNT);
    let img = channel_shift(img, CHANNEL_SHIFT_AMOUNT);
    //let img = pixel_blur(img, PIXEL_BLUR_AMOUNT, PIXEL_BLUR_FLUCTUATION);
    let img = granular(img);
    let img = tear(img, TEAR_MAX_TIMES, TEAR_MAX_HEIGHT, TEAR_AMOUNT, rng);
    let img = noise(img, 4);

    img.save("output.png").unwrap();
}

fn channel_shift(
    input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>, amount: u32,
) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("ChannelShift FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();

    let amount = (amount as f32 * SCALE).floor() as u32;

    for x in amount..(width - amount) {
        for y in 0..height {
            let [left_r, _, _, _] = input.get_pixel(x + amount, y).0;
            let [_, _, right_b, _] = input.get_pixel(x - amount, y).0;
            let [r, g, b, a] = input.get_pixel(x, y).0;

            let new_r = cmp::max(r, left_r);
            let new_b = cmp::max(b, right_b);

            output.put_pixel(x, y, image::Rgba([new_r, g, new_b, a]));
        }
    }

    output
}

fn ghost(
    input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>, amount: u32
) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("Ghost FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();

    let amount = (amount as f32 * SCALE).floor() as u32;

    for x in amount..width {
        for y in 0..height {
            let [_, src_g, _, _] = input.get_pixel(x - amount, y).0;
            let [r, g, b, a] = input.get_pixel(x, y).0;

            let dst_g = cmp::max(b, src_g);

            output.put_pixel(x, y, image::Rgba([r, dst_g, b, a]));
        }
    }

    output
}
/*
fn continurous_ghost(input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("ContinurousGhost FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();

    for x in 0..width {
        for y in 0..height {
            let [_, src_g, _, _] = input.get_pixel(x - amount, y).0;
            let [r, g, b, a] = input.get_pixel(x, y).0;

            let dst_g = cmp::max(b, src_g);

            output.put_pixel(x, y, image::Rgba([r, dst_g, b, a]));
        }
    }

    output
}*/

fn tear(
    input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
    max_times: u32,
    max_height: u32,
    amount: u32,
    mut rng: rand_xoshiro::Xoshiro256StarStar,
) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("Tear FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();

    let shift_times = (rng.gen::<f64>() * max_times as f64).floor() as u64;
    for _ in 0..shift_times {
        let begin_y = ((rng.gen::<f64>() * height as f64) * SCALE as f64).floor() as u32;
        let shift_height = ((rng.gen::<f64>() * max_height as f64) * SCALE as f64).floor() as u32;
        let shift_amount = ((rng.gen::<f64>() * amount as f64) * SCALE as f64).floor() as u32;

        for x in shift_amount..(width - shift_amount) {
            let max_y = cmp::min(height, begin_y + shift_height);
            for y in begin_y..max_y {
                // TODO: 左右どちらにシフトするかをランダムにする
                output.put_pixel(x, y, *input.get_pixel(x - shift_amount, y));
            }
        }
    }

    output
}

fn pixel_blur(input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>, amount: u32, fluctuation: u32) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("PixelBlur FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();
    let mut rng = rand::thread_rng();
    let fade = true;

    for x in 0..width {
        for y in 0..height {
            let _y = height - y - 1;
            let [r, g, b, a] = input.get_pixel(x, _y).0;
            let intensity = ((r as f32) + (g as f32) + (b as f32)) / (255 * 3) as f32;
            let velocity = (intensity * amount as f32).floor() as u32;
            let velocity = velocity + rng.gen_range(0, fluctuation);
            if fade {
                for i in 0..velocity {
                    if _y + i < height {
                        output.put_pixel(x, _y + i, blend(
                            *input.get_pixel(x, _y + i),
                            image::Rgba([r, g, b, a]),
                            (i as f32) / (velocity as f32)));
                    }
                }
            } else {
                for i in 0..velocity {
                    if _y + i < height {
                        output.put_pixel(x, _y + i, image::Rgba([r, g, b, a]));
                    }
                }
            }
        }
    }

    output
}

fn granular(input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("Granular FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();
    let times = 1024;
    let granular_width = 32 * SCALE as i32;
    let granular_height = 8 * SCALE as i32;
    let granular_velocity = 32 * SCALE as i32;
    let mut rng = rand::thread_rng();

    for _ in 0..times {
        let src_x: i32 = rng.gen_range(0, width) as i32;
        let src_y: i32 = rng.gen_range(0, height) as i32;
        let dst_x: i32 = src_x + (rng.gen_range(0, granular_velocity * 2) - granular_velocity);
        let dst_y: i32 = src_y + (rng.gen_range(0, granular_velocity * 2) - granular_velocity);

        for x in 0..granular_width {
            for y in 0..granular_height {
                let dst_x = dst_x + x;
                let dst_y = dst_y + y;
                let x = x + src_x;
                let y = y + src_y;

                if (x >= width as i32) || (x < 0) { continue; }
                if (y >= height as i32) || (y < 0) { continue; }
                if (dst_x >= width as i32) || (dst_x < 0) { continue; }
                if (dst_y >= height as i32) || (dst_y < 0) { continue; }

                let [r, g, b, a] = input.get_pixel(x as u32, y as u32).0;

                output.put_pixel(dst_x as u32, dst_y as u32, image::Rgba([r, g, b, a]));
            }
        }
    }

    output
}

// TODO: SCALE support
fn noise(
    input: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
    size: u32,
) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
    println!("Noise FX");

    let mut output = input.clone();
    let (width, height) = input.dimensions();
    let times = 2048;
    let velocity = 128;
    let mut rng = rand::thread_rng();

    for _ in 0..times {
        let noise_x = rng.gen_range(0, width);
        let noise_y = rng.gen_range(0, height);
        let px = input.get_pixel(noise_x, noise_y);

        for i in 0..velocity {
            let dst_x = noise_x + i;
            let dst_y = noise_y;

            if dst_x >= width { continue; }

            for j in 0..size {
                if dst_y + j >= height { continue; }

                output.put_pixel(dst_x, dst_y + j, blend(
                    image::Rgba(input.get_pixel(dst_x, dst_y).0),
                    image::Rgba(px.0),
                    1 as f32 - (i as f32 / velocity as f32) as f32
                ));
            }
        }
    }

    output
}

fn blend(dst: Rgba<u8>, src: Rgba<u8>, mix: f32) -> image::Rgba<u8> {
    let mix: u16 = (mix * (255 as f32)).floor() as u16;
    let [r1, g1, b1, a1] = cast_array!(src.0, u16, 4);
    let [r2, g2, b2, a2] = cast_array!(dst.0, u16, 4);
    let r = (r1 * mix + r2 * (255 - mix)) / 255;
    let g = (g1 * mix + g2 * (255 - mix)) / 255;
    let b = (b1 * mix + b2 * (255 - mix)) / 255;
    let a = (a1 * mix + a2 * (255 - mix)) / 255;
    image::Rgba([r as u8, g as u8, b as u8, a as u8])
}
