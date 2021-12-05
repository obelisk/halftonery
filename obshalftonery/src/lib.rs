
use obs_wrapper::{obs_register_module, obs_string, prelude::*, source::*};
use obs_wrapper::source::video::VideoFormat;

use imageproc::drawing::Canvas;

use halftonery::{
    color,
    process_image_from_cmyk_buffers
};

struct Data {
    spacing: u32,
}

struct CmykBuffers {
    c: Vec<f64>,
    m: Vec<f64>,
    y: Vec<f64>,
    k: Vec<f64>,
}

struct HalftoneryFilter {
    context: ModuleContext,
}

fn clamp(num: i32) -> u8 {
    if num > 255 {
        255
    } else if num < 0 {
        0
    } else {
        num as u8
    }
}

fn convert_uyuv_frame_to_cmyk_buffers(video: &video::VideoDataContext) -> CmykBuffers {
    let p = video.get_data_buffer(0);
    let mut offset = 0;
    let height = video.get_height() as usize;
    let width = video.get_width() as usize;

    let mut c_buf = Vec::with_capacity(width * height);
    let mut m_buf = Vec::with_capacity(width * height);
    let mut y_buf = Vec::with_capacity(width * height);
    let mut k_buf = Vec::with_capacity(width * height);

    c_buf.resize(width * height, 0.0);
    m_buf.resize(width * height, 0.0);
    y_buf.resize(width * height, 0.0);
    k_buf.resize(width * height, 0.0);

    for h in 0..height {
        let mut w = 0;
        for _ in 0..width/2 {
            let u0 = unsafe { *p.offset(offset) as i32 };
            let y0 = unsafe { *p.offset(offset + 1) as i32 };
            let v0 = unsafe { *p.offset(offset + 2) as i32 };
            let y1 = unsafe { *p.offset(offset + 3) as i32 };
            offset += 4;

            let c = y0 - 16;
            let d = u0 - 128;
            let e = v0 - 128;
            
            let px1 = color::convert_rgb_to_cmyk(&color::Rgb {
                r: clamp((298 * c + 409 * e + 128) >> 8),
                g: clamp((298 * c - 100 * d - 208 * e + 128) >> 8),
                b: clamp((298 * c + 516 * d + 128) >> 8),
            });

            // h = height we're on, w width we're on vs width being total width
            c_buf[h as usize * width + w as usize] = px1.c;
            m_buf[h as usize * width + w as usize] = px1.m;
            y_buf[h as usize * width + w as usize] = px1.y;
            k_buf[h as usize * width + w as usize] = px1.k;  

            w += 1;
            let c = y1 - 16;

            let px2 = color::convert_rgb_to_cmyk(&color::Rgb {
                r: clamp((298 * c + 409 * e + 128) >> 8),
                g: clamp((298 * c - 100 * d - 208 * e + 128) >> 8),
                b: clamp((298 * c + 516 * d + 128) >> 8),
            });

            c_buf[h as usize * width + w as usize] = px2.c;
            m_buf[h as usize * width + w as usize] = px2.m;
            y_buf[h as usize * width + w as usize] = px2.y;
            k_buf[h as usize * width + w as usize] = px2.k;  

            w += 1;
        }
    }

    CmykBuffers {
        c: c_buf,
        m: m_buf,
        y: y_buf,
        k: k_buf,
    }
}

impl Sourceable for HalftoneryFilter {
    fn get_id() -> ObsString {
        obs_string!("halftonery_video_filter")
    }
    fn get_type() -> SourceType {
        SourceType::FILTER
    }
}

impl GetNameSource<Data> for HalftoneryFilter {
    fn get_name() -> ObsString {
        obs_string!("Halftone Filter")
    }
}

impl CreatableSource<Data> for HalftoneryFilter {
    fn create(_create: &mut CreatableSourceContext<Data>, mut _source: SourceContext) -> Data {
        Data {
            spacing: 12,
        }
    }
}

impl UpdateSource<Data> for HalftoneryFilter {
    fn update(data: &mut Option<Data>, settings: &mut DataObj, _context: &mut GlobalContext) {
        if let Some(data) = data {
            if let Some(spacing) = settings.get(obs_string!("spacing")) {
                data.spacing = spacing;
            }
        }
    }
}

impl GetPropertiesSource<Data> for HalftoneryFilter {
    fn get_properties(_data: &mut Option<Data>, properties: &mut Properties) {
        properties
            .add(
                obs_string!("spacing"),
                obs_string!("Dot Spacing"),
                NumberProp::new_int().with_range(4u32..=200),
            );
    }
}


impl FilterVideoSource<Data> for HalftoneryFilter {
    fn filter_video(data: &mut Option<Data>, video: &mut video::VideoDataContext) {
        if video.get_format() != VideoFormat::UYVY {
            println!("Only the UYVY colour format is currently supported. Found: {:?}", video.get_format());
            return;
        }
        let bufs = convert_uyuv_frame_to_cmyk_buffers(video);
        let width = video.get_width() as usize;
        let height = video.get_height() as usize;
        let spacing = if let Some(d) = data {d.spacing} else { 12 };

        let processed_image = process_image_from_cmyk_buffers(width, height, spacing, &bufs.c, &bufs.m, &bufs.y, &bufs.k);

        let p = video.get_data_buffer(0);
        let linesize = video.get_linesize(0) as usize;

        for y in 0..height {
            for x in 0..width {
                let pixel = processed_image.get_pixel(x as u32, y as u32);
                let r = pixel[0] as f64;
                let g = pixel[1] as f64;
                let b = pixel[2] as f64;

                let c_y = clamp(((0.257*r) + (0.504*g) + (0.098*b) + 16.) as i32);
                let c_u = clamp((-(0.148*r) - (0.291*g) + (0.439*b) + 128.) as i32);
                let c_v = clamp(((0.439*r) - (0.368*g) - (0.071*b) + 128.) as i32);

                unsafe {
                    if x & 1 == 0 {
                        *p.offset((y * linesize + x * 2 + 1) as isize) = c_y;
                        *p.offset((y * linesize + x * 2 + 2) as isize) = c_u;
                        *p.offset((y * linesize + x * 2 + 0) as isize) = c_v;
                    } else {
                        *p.offset((y * linesize + x * 2 + 1) as isize) = c_y;
                        *p.offset((y * linesize + x * 2 - 2) as isize) = c_u;
                        *p.offset((y * linesize + x * 2 + 0) as isize) = c_v;
                    }
                }
            }
        }
    }
}

impl Module for HalftoneryFilter {
    fn new(context: ModuleContext) -> Self {
        Self { context }
    }
    fn get_ctx(&self) -> &ModuleContext {
        &self.context
    }

    fn load(&mut self, load_context: &mut LoadContext) -> bool {
        let source = load_context
            .create_source_builder::<HalftoneryFilter, Data>()
            .enable_get_name()
            .enable_get_properties()
            .enable_create()
            .enable_update()
            .enable_filter_video()
            .build();

        load_context.register_source(source);

        true
    }

    fn description() -> ObsString {
        obs_string!("A filter that calculates and applies a halftoning to a video source.")
    }
    fn name() -> ObsString {
        obs_string!("Halftone Filter")
    }
    fn author() -> ObsString {
        obs_string!("Mitchell Grenier")
    }
}

obs_register_module!(HalftoneryFilter);
