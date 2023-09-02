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
const FAR_LEFT: u32 = 160;
const FAR_TOP: u32 = 1000;

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

fn draw_text(text: &str) -> Vec<u8> {
    let width = write_to_crop(text);

    let mut left = FAR_LEFT;
    let mut top = FAR_TOP;
    let mut font_size = MAX_FONT_SIZE;

    if width < MAX_WIDTH {
        left = FAR_LEFT + (MAX_WIDTH - width) / 2;
    } else {
        font_size = (MAX_WIDTH as f32) / (width as f32) * MAX_FONT_SIZE;
        top = FAR_TOP + ((1.0 - (MAX_WIDTH as f32) / (width as f32)) * (MAX_HEIGHT as f32)) as u32;
    }

    let mut img = image::load_from_memory(TEMPLATE_BUF).unwrap();
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

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    request_received(handler).await;
}

async fn handler(qry: HashMap<String, Value>, _body: Vec<u8>) {
    let text = qry.get("text").expect("No text provided").as_str().unwrap();

    let image = draw_text(text);
    let image = base64::encode_config(image, base64::STANDARD);
    send_response(
        200,
        vec![
            (String::from("content-type"), String::from("image/png")),
            (
                String::from("Access-Control-Allow-Origin"),
                String::from("*"),
            ),
        ],
        image.as_bytes().to_vec(),
    );
}
