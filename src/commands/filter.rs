use std::io::{BufWriter, Cursor};

use eframe::egui::{Slider, Ui, Widget};
use image::{
    DynamicImage, GenericImage, GenericImageView, Rgba,
    codecs::jpeg::{JpegDecoder, JpegEncoder},
};

#[derive(Debug, Clone)]
pub enum ImageFilter {
    JpegCompression { quality: u8 },
    Saturate { percentage: u8 },
}

impl ImageFilter {
    pub const DEFAULTS: &[ImageFilter] = &[
        Self::JpegCompression { quality: 80 },
        Self::Saturate { percentage: 100 },
    ];

    pub const NAMES: &[&str] = &["JPEG Compression", "Saturate"];

    pub fn name(&self) -> &str {
        match self {
            Self::JpegCompression { .. } => Self::NAMES[0],
            Self::Saturate { .. } => Self::NAMES[1],
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self {
            Self::JpegCompression { quality } => {
                Slider::new(quality, 1..=100).text("Quality").ui(ui);
            }
            Self::Saturate { percentage } => {
                Slider::new(percentage, 0..=200)
                    .text("Saturation (%)")
                    .ui(ui);
            }
        }
    }

    // TODO: error management
    // FIXME: This is not how you saturate an image
    pub fn apply(&self, img: DynamicImage) -> DynamicImage {
        match self {
            Self::JpegCompression { quality } => {
                let mut buf = BufWriter::new(Vec::new());
                let encoder = JpegEncoder::new_with_quality(&mut buf, *quality);
                img.write_with_encoder(encoder).unwrap();
                let bytes = buf.into_inner().unwrap();
                let decoder = JpegDecoder::new(Cursor::new(bytes)).unwrap();
                DynamicImage::from_decoder(decoder).unwrap()
            }
            Self::Saturate { percentage } => {
                let mut img = img;
                let percent = *percentage as f32 / 100.;
                for (x, y, col) in img.clone().pixels() {
                    let [r, g, b, a] = col.0;
                    let r = (r as f32 * percent).clamp(0., 255.) as u8;
                    let g = (g as f32 * percent).clamp(0., 255.) as u8;
                    let b = (b as f32 * percent).clamp(0., 255.) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
                img
            }
        }
    }
}
