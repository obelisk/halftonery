mod color;

use halftonery::process_image_at_path;

use image::GenericImageView;
use image::{Rgba, RgbaImage};
use imageproc::rect::Rect;
use imageproc::drawing::{
    draw_filled_rect_mut,
    draw_filled_circle_mut
};
use imageproc::drawing::{Blend};
use image::GenericImage;

fn main() {
    println!("Halftonery");
    let args: Vec<String> = std::env::args().collect();
    let input_path = &args[1];
    let spacing = args[2].parse::<u32>().unwrap_or(16);
    let output_path = format!("{}_halftoned_at_{}.png", &input_path[..input_path.len() - 4], spacing);
    process_image_at_path(&input_path, &output_path, spacing).unwrap();
}

