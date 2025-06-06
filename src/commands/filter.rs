use std::io::{BufWriter, Cursor};

use eframe::egui::{Slider, Ui, Widget};
use image::{
    DynamicImage, GenericImage, GenericImageView, Rgba,
    codecs::jpeg::{JpegDecoder, JpegEncoder},
};

#[derive(Debug, Clone)]
pub enum ImageFilter {
    JpegCompression { quality: u8 },
    Brightness { percentage: u8 },
    Sharpen,
    BoxBlur,
    GaussianBlur { sigma: f32 },
}

impl ImageFilter {
    pub const DEFAULTS: &[ImageFilter] = &[
        Self::JpegCompression { quality: 80 },
        Self::Brightness { percentage: 100 },
        Self::Sharpen,
        Self::BoxBlur,
        Self::GaussianBlur { sigma: 2. },
    ];

    pub const NAMES: &[&str] = &[
        "JPEG Compression",
        "Brightness",
        "Sharpen",
        "Box Blur",
        "Gaussian Blur",
    ];

    pub fn name(&self) -> &str {
        match self {
            Self::JpegCompression { .. } => Self::NAMES[0],
            Self::Brightness { .. } => Self::NAMES[1],
            Self::Sharpen => Self::NAMES[2],
            Self::BoxBlur => Self::NAMES[3],
            Self::GaussianBlur { .. } => Self::NAMES[4],
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self {
            Self::JpegCompression { quality } => {
                Slider::new(quality, 1..=100).text("Quality").ui(ui);
            }
            Self::Brightness { percentage } => {
                Slider::new(percentage, 0..=200)
                    .text("Brightness (%)")
                    .ui(ui);
            }
            Self::GaussianBlur { sigma } => {
                Slider::new(sigma, 1.0..=5.0).text("Blur Variance").ui(ui);
            }
            Self::Sharpen | Self::BoxBlur => {}
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
            Self::Brightness { percentage } => {
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
            Self::Sharpen => img.filter3x3(&[0., -1., 0., -1., 5., -1., 0., -1., 0.]),
            Self::BoxBlur => {
                let n = 1. / 9.;
                img.filter3x3(&[n, n, n, n, n, n, n, n, n])
            }
            Self::GaussianBlur { sigma } => img.blur(*sigma),
        }
    }
}
