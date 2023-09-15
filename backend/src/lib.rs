use std::collections::HashMap;

use lambda_flows::{request_received, send_response};
use serde_json::Value;

use image;
use imageproc::drawing;
use rusttype::{Font, Scale};
mod imagecrop;

const MAX_WIDTH: u32 = 600;
const MAX_HEIGHT: u32 = 80;
const MAX_FONT_SIZE: f32 = 125.0;

const FONT_FILE: &[u8] = include_bytes!("PingFang-Bold.ttf") as &[u8];
const TEMPLATE_BUF: &[u8] = include_bytes!("template.png") as &[u8];

fn write_to_crop(watermark_text: &str) -> u32 {
    let buffer = include_bytes!("crop.png") as &[u8];
    let mut img = image::load_from_memory(&buffer.to_vec()).unwrap();

    let scale = Scale {
        x: MAX_FONT_SIZE,
        y: MAX_FONT_SIZE,
    };

    let font = Vec::from(FONT_FILE);
    let font = Font::try_from_vec(font).unwrap();

    drawing::draw_text_mut(
        &mut img,
        image::Rgba([0u8, 0u8, 0u8, 1u8]),
        0,
        0,
        scale,
        &font,
        watermark_text,
    );

    let ic = imagecrop::ImageCrop::new(img).unwrap();
    let corners = ic.calculate_corners();
    return corners.1.x - corners.0.x;
}

fn draw_text(mut img: image::DynamicImage, text: &str, far_left: u32, far_top: u32) -> Vec<u8> {
    let width = write_to_crop(text);

    let mut left = far_left;
    let mut top = far_top;
    let mut font_size = MAX_FONT_SIZE;

    if width < MAX_WIDTH {
        left = far_left + (MAX_WIDTH - width) / 2;
    } else {
        font_size = (MAX_WIDTH as f32) / (width as f32) * MAX_FONT_SIZE;
        top = far_top + ((1.0 - (MAX_WIDTH as f32) / (width as f32)) * (MAX_HEIGHT as f32)) as u32;
    }

    let scale = Scale {
        x: font_size,
        y: font_size,
    };

    let font = Vec::from(FONT_FILE);
    let font = Font::try_from_vec(font).unwrap();

    drawing::draw_text_mut(
        &mut img,
        image::Rgba([255u8, 255u8, 255u8, 1u8]),
        left,
        top,
        scale,
        &font,
        text,
    );

    let mut buf = vec![];
    img.write_to(&mut buf, image::ImageOutputFormat::Png)
        .unwrap();
    return buf;
}

fn draw_filled_circle_mut<C>(canvas: &mut C, center: (i32, i32), radius: i32, color: C::Pixel)
where
    C: drawing::Canvas,
    C::Pixel: 'static,
{
    let mut x = 0i32;
    let mut y = radius;
    let mut p = 1 - radius;
    let x0 = center.0;
    let y0 = center.1;

    while x <= y {
        // drawing::draw_line_segment_mut(
        //     canvas,
        //     ((x0 - x) as f32, (y0 + y) as f32),
        //     ((x0 + x) as f32, (y0 + y) as f32),
        //     color,
        // );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 - radius) as f32, (y0 + y) as f32),
            ((x0 - x) as f32, (y0 + y) as f32),
            color,
        );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 + x) as f32, (y0 + y) as f32),
            ((x0 + radius) as f32, (y0 + y) as f32),
            color,
        );

        // drawing::draw_line_segment_mut(
        //     canvas,
        //     ((x0 - y) as f32, (y0 + x) as f32),
        //     ((x0 + y) as f32, (y0 + x) as f32),
        //     color,
        // );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 - radius) as f32, (y0 + x) as f32),
            ((x0 - y) as f32, (y0 + x) as f32),
            color,
        );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 + y) as f32, (y0 + x) as f32),
            ((x0 + radius) as f32, (y0 + x) as f32),
            color,
        );

        // drawing::draw_line_segment_mut(
        //     canvas,
        //     ((x0 - x) as f32, (y0 - y) as f32),
        //     ((x0 + x) as f32, (y0 - y) as f32),
        //     color,
        // );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 - radius) as f32, (y0 - y) as f32),
            ((x0 - x) as f32, (y0 - y) as f32),
            color,
        );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 + x) as f32, (y0 - y) as f32),
            ((x0 + radius) as f32, (y0 - y) as f32),
            color,
        );

        // drawing::draw_line_segment_mut(
        //     canvas,
        //     ((x0 - y) as f32, (y0 - x) as f32),
        //     ((x0 + y) as f32, (y0 - x) as f32),
        //     color,
        // );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 - radius) as f32, (y0 - x) as f32),
            ((x0 - y) as f32, (y0 - x) as f32),
            color,
        );
        drawing::draw_line_segment_mut(
            canvas,
            ((x0 + y) as f32, (y0 - x) as f32),
            ((x0 + radius) as f32, (y0 - x) as f32),
            color,
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

fn draw_avatar(body: Vec<u8>, width: u32, height: u32, left: u32, top: u32) -> image::DynamicImage {
    let mut avatar = image::load_from_memory(&body).unwrap();
    avatar = avatar.resize(width, height, image::imageops::Lanczos3);

    {
        let mut png_buf = std::io::Cursor::new(Vec::new());
        avatar
            .write_to(&mut png_buf, image::ImageOutputFormat::Png)
            .unwrap();
        avatar = image::load_from_memory(&png_buf.into_inner()).unwrap();
    }

    let x = width as i32 / 2;
    draw_filled_circle_mut(&mut avatar, (x, x), x, image::Rgba([0 as u8; 4]));

    let mut img = image::load_from_memory(TEMPLATE_BUF).unwrap();
    image::imageops::overlay(&mut img, &avatar, left, top);
    img
}

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    request_received(handler).await;
}

fn get_qry(qry: &HashMap<String, Value>, key: &str, default: u32) -> u32 {
    qry.get(key)
        .unwrap_or(&Value::from(""))
        .as_str()
        .unwrap_or("")
        .parse()
        .unwrap_or(default)
}

async fn handler(qry: HashMap<String, Value>, body: Vec<u8>) {
    let text = qry.get("text").expect("No text provided").as_str().unwrap();

    let aw = get_qry(&qry, "aw", 0);
    let ah = get_qry(&qry, "ah", 0);
    let al = get_qry(&qry, "al", 0);
    let at = get_qry(&qry, "at", 0);

    let tl = get_qry(&qry, "tl", 0);
    let tt = get_qry(&qry, "tt", 0);

    let image = draw_avatar(body, aw, ah, al, at);
    let image_buf = draw_text(image, text, tl, tt);
    send_response(
        200,
        vec![
            (String::from("content-type"), String::from("image/png")),
            (
                String::from("Access-Control-Allow-Origin"),
                String::from("*"),
            ),
        ],
        image_buf,
    );
}
