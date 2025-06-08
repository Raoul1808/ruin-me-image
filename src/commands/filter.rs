use std::io::{BufWriter, Cursor};

use eframe::egui::{
    DragValue, Slider, Ui, Widget,
    ecolor::{hsv_from_rgb, rgb_from_hsv},
};
use image::{
    DynamicImage, GenericImage, GenericImageView, Pixel, Rgba,
    codecs::jpeg::{JpegDecoder, JpegEncoder},
};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFilter {
    JpegCompression { quality: u8 },
    Brightness { percentage: u8 },
    Sharpen,
    BoxBlur,
    GaussianBlur { sigma: f32 },
    Saturate { percentage: u16 },
    Noise { strength: u8, seed: Option<u64> },
}

impl ImageFilter {
    pub const DEFAULTS: &[ImageFilter] = &[
        Self::JpegCompression { quality: 80 },
        Self::Brightness { percentage: 100 },
        Self::Sharpen,
        Self::BoxBlur,
        Self::GaussianBlur { sigma: 2. },
        Self::Saturate { percentage: 100 },
        Self::Noise {
            strength: 10,
            seed: None,
        },
    ];

    pub const NAMES: &[&str] = &[
        "JPEG Compression",
        "Brightness",
        "Sharpen",
        "Box Blur",
        "Gaussian Blur",
        "Saturate",
        "Noise",
    ];

    pub fn name(&self) -> &str {
        match self {
            Self::JpegCompression { .. } => Self::NAMES[0],
            Self::Brightness { .. } => Self::NAMES[1],
            Self::Sharpen => Self::NAMES[2],
            Self::BoxBlur => Self::NAMES[3],
            Self::GaussianBlur { .. } => Self::NAMES[4],
            Self::Saturate { .. } => Self::NAMES[5],
            Self::Noise { .. } => Self::NAMES[6],
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
                Slider::new(sigma, 0.0..=5.0).text("Blur Variance").ui(ui);
            }
            Self::Saturate { percentage } => {
                Slider::new(percentage, 0..=400)
                    .text("Saturation (%)")
                    .ui(ui);
            }
            Self::Noise { strength, seed } => {
                let seed_number = seed.unwrap_or(rand::random());
                ui.horizontal(|ui| {
                    ui.radio_value(seed, None, "Random");
                    ui.radio_value(seed, Some(seed_number), "Seeded");
                });
                if let Some(s) = seed {
                    ui.horizontal(|ui| {
                        ui.label("Seed");
                        DragValue::new(s).speed(10).ui(ui);
                        if ui.button("Random Seed").clicked() {
                            *s = rand::random();
                        }
                    });
                }
                Slider::new(strength, 0..=100).text("Noise Strength").ui(ui);
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
                img.to_rgb8().write_with_encoder(encoder).unwrap();
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
            Self::Saturate { percentage } => {
                let mut img = img;
                let percent = *percentage as f32 / 100.;
                for (x, y, col) in img.clone().pixels() {
                    let [r, g, b, a] = col.0;
                    let (h, s, v) =
                        hsv_from_rgb([r as f32 / 255., g as f32 / 255., b as f32 / 255.]);
                    let s = (s * percent).clamp(0.0, 1.0);
                    let [r, g, b] = rgb_from_hsv((h, s, v));
                    let r = (r * 255.) as u8;
                    let g = (g * 255.) as u8;
                    let b = (b * 255.) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
                img
            }
            Self::Noise { strength, seed } => {
                let mut img = img;
                let mut random =
                    rand_chacha::ChaCha20Rng::seed_from_u64(seed.unwrap_or(rand::random()));
                let percent = *strength as f32 / 100.;
                for (x, y, col) in img.clone().pixels() {
                    let rnoise = random.random_range(0.0..=1.0);
                    let noise = 1.0 - (rnoise * percent);
                    let col = col.map_without_alpha(|c| (((c as f32 / 255.) * noise) * 255.) as u8);
                    img.put_pixel(x, y, col);
                }
                img
            }
        }
    }
}
