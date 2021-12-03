mod color;

use image::GenericImageView;
use image::{Rgb, RgbImage};
use imageproc::rect::Rect;
use imageproc::drawing::{
    draw_filled_rect_mut,
    draw_filled_circle_mut
};
use imageproc::drawing::{Blend};
use image::GenericImage;

const CMYK_ANGLES: [f64; 4] = [15., 75., 90., 45.];

fn calculate_intensity_at_point(x: u32, y: u32, width: usize, height: usize, spacing: u32, color_buf: &[f64]) -> f64 {
    // Convert to signed values to allow ranges below 0 to handle literal edge cases
    let x = x as i32;
    let y = y as i32;
    let spacing = spacing as i32;
    // Sum up in range pixels
    let mut aggregate_pixel_value = 0.;
    let mut pixels_in_range = 0;
    
    for x in (x - spacing / 2)..(x + spacing / 2) {
        for y in (y - spacing / 2)..(y + spacing / 2) {
            if x < 0 || y < 0 || x as usize > width - 1 || y as usize > height - 1 {
                continue;
            }
            aggregate_pixel_value += color_buf[y as usize * width + x as usize];
            pixels_in_range += 1;
        }
    }

    aggregate_pixel_value as f64 / pixels_in_range as f64
}

// This loops infinitely if angle is 0 because none of the checks hit.
// Fix this
fn calculate_dots(deg_angle: f64, width: usize, height: usize, spacing: u32, color_buf: &[f64]) -> Vec<(u32, u32, f64)> {
    let width = width as i32;
    let height = height as i32;
    let mut locations = Vec::with_capacity(2000);
    let angle = -1.0 * deg_angle * 2. * std::f64::consts::PI / 360 as f64;

    let x_spacing = (spacing as f64 * angle.cos()) as i32;
    let y_spacing = (spacing as f64 * angle.sin()) as i32;

    let x_newline_spacing = ((spacing as f64) * (std::f64::consts::PI / 2.0 - angle).cos()) as i32;
    let y_newline_spacing = ((spacing as f64) * (std::f64::consts::PI / 2.0 - angle).sin()) as i32;

    let mut x_coord = 0;
    let mut y_coord = 0;
    let mut line = 0;
    
    loop {
        // Create dots off to the right
        let mut x_coord_r = x_coord;
        let mut y_coord_r = y_coord;
        loop {
            if x_coord_r >= width || y_coord_r <= 0 as i32 {
                break;
            }
            if y_coord_r >= 0 as i32 && y_coord_r < height as i32 && x_coord_r < width && x_coord_r >= 0   {
                let intensity = calculate_intensity_at_point(x_coord_r as u32, y_coord_r as u32, width as usize, height as usize, spacing, color_buf);
                locations.push((x_coord_r as u32, y_coord_r as u32, intensity));
            }

            x_coord_r += x_spacing;
            y_coord_r += y_spacing;
        }

        // Create dots off to the left
        let mut x_coord_l = x_coord;
        let mut y_coord_l = y_coord;
        loop {
            if x_coord_l <= 0 as i32 || y_coord_l >= height {
                break;
            }

            if y_coord_l >= 0 as i32 && y_coord_l < height as i32 && x_coord_l < width && x_coord_l >= 0 {
                let intensity = calculate_intensity_at_point(x_coord_l as u32, y_coord_l as u32, width as usize, height as usize, spacing, color_buf);
                locations.push((x_coord_l as u32, y_coord_l as u32, intensity));
            }

            x_coord_l -= x_spacing;
            y_coord_l -= y_spacing;
        }

        //img_dots.put_pixel(x_coord as u32, y_coord as u32, Rgb([255, 255, 255]));
        // Go to the next line
        line += 1;
        x_coord = -x_newline_spacing * line;
        y_coord = y_newline_spacing * line;
        if x_coord > (width*2 - 1) as i32 || y_coord < 0 as i32 {
            break;
        }
    }
    locations
}


fn main() {
    println!("Halftonery");
    let args: Vec<String> = std::env::args().collect();
    let input_path = &args[1];
    let spacing = args[2].parse::<u32>().unwrap_or(16);
    let output_path = format!("{}_halftoned_at_{}.png", &input_path[..input_path.len() - 3], spacing);

    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open(input_path).unwrap();

    let width = img.width() as usize;
    let height = img.height() as usize;

    // Debug code for holding separate layer images
    /*
    let mut img_c: RgbImage = ImageBuffer::new(img.width(), img.height());
    let mut img_m: RgbImage = ImageBuffer::new(img.width(), img.height());
    let mut img_y: RgbImage = ImageBuffer::new(img.width(), img.height());
    let mut img_k: RgbImage = ImageBuffer::new(img.width(), img.height());
    */

    let mut c_buf = Vec::with_capacity(width * height);
    let mut m_buf = Vec::with_capacity(width * height);
    let mut y_buf = Vec::with_capacity(width * height);
    let mut k_buf = Vec::with_capacity(width * height);

    c_buf.resize(width * height, 0.0);
    m_buf.resize(width * height, 0.0);
    y_buf.resize(width * height, 0.0);
    k_buf.resize(width * height, 0.0);

    // Iterate over all pixels in the image.
    for (x, y, rgba) in img.pixels() {
        let cmyk = color::convert_rgb_to_cmyk(&color::Rgb{r: rgba[0], g: rgba[1], b: rgba[2]});

        /*
        let c_channel = color::convert_cmyk_to_rgb(&color::Cmyk{c: cmyk.c, m: 0., y: 0., k: 0.,});
        let m_channel = color::convert_cmyk_to_rgb(&color::Cmyk{c: 0., m: cmyk.m, y: 0., k: 0.,});
        let y_channel = color::convert_cmyk_to_rgb(&color::Cmyk{c: 0., m: 0., y: cmyk.y, k: 0.,});
        let k_channel = color::convert_cmyk_to_rgb(&color::Cmyk{c: 0., m: 0., y: 0., k: cmyk.k,});
        */

        c_buf[y as usize * width + x as usize] = cmyk.c;
        m_buf[y as usize * width + x as usize] = cmyk.m;
        y_buf[y as usize * width + x as usize] = cmyk.y;
        k_buf[y as usize * width + x as usize] = cmyk.k;
        
        // Debug code for building the separate layer images
        /*
        img_c.put_pixel(x, y, Rgb([c_channel.r, c_channel.g, c_channel.b]));
        img_m.put_pixel(x, y, Rgb([m_channel.r, m_channel.g, m_channel.b]));
        img_y.put_pixel(x, y, Rgb([y_channel.r, y_channel.g, y_channel.b]));
        img_k.put_pixel(x, y, Rgb([k_channel.r, k_channel.g, k_channel.b]));
        */
    }

    // Debug: Outputing completed cmyk layers
    /*
    img_c.save("hillside_c.jpg").unwrap();
    img_m.save("hillside_m.jpg").unwrap();
    img_y.save("hillside_y.jpg").unwrap();
    img_k.save("hillside_k.jpg").unwrap();
    */

    // Quantize
    //let spacing = 8;
    let c_locations = calculate_dots(CMYK_ANGLES[0], width, height, spacing, &c_buf);
    let m_locations = calculate_dots(CMYK_ANGLES[1], width, height, spacing, &m_buf);
    let y_locations = calculate_dots(CMYK_ANGLES[2], width, height, spacing, &y_buf);
    let k_locations = calculate_dots(CMYK_ANGLES[3], width, height, spacing, &k_buf);

    //let mut output = RgbaImage::new(width as u32, height as u32);
    let mut output = Blend(RgbImage::new(width as u32, height as u32));

    draw_filled_rect_mut(&mut output,  Rect::at(0, 0).of_size(width as u32, height as u32), Rgb([255u8, 255u8, 255u8]));

    let spacing = spacing as f64 / 1.8;
    
    for c in k_locations.iter() {
        draw_filled_circle_mut(&mut output, (c.0 as i32, c.1 as i32), ((spacing) as f64 * c.2) as i32, Rgb([0u8, 0u8, 0u8]));
    }

    for c in c_locations.iter() {
        draw_filled_circle_mut(&mut output, (c.0 as i32, c.1 as i32), ((spacing) as f64 * c.2) as i32, Rgb([0u8, 255u8, 255u8]));
    }
    
    for c in m_locations.iter() {
        draw_filled_circle_mut(&mut output, (c.0 as i32, c.1 as i32), ((spacing) as f64 * c.2) as i32, Rgb([255u8, 0u8, 255u8]));
    }

    for c in y_locations.iter() {
        draw_filled_circle_mut(&mut output, (c.0 as i32, c.1 as i32), ((spacing as f64 / 1.0) as f64 * c.2) as i32, Rgb([255u8, 255u8, 0u8]));
    }

    output.0.inner_mut().save(output_path).unwrap();
}
