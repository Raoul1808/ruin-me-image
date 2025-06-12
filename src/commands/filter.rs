use std::io::{BufWriter, Cursor};

use eframe::egui::{
    DragValue, RadioButton, Slider, Ui, Widget,
    ecolor::{hsv_from_rgb, rgb_from_hsv},
};
use image::{
    DynamicImage, GenericImage, GenericImageView, Pixel, Rgba,
    codecs::jpeg::{JpegDecoder, JpegEncoder},
};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResizeOption {
    Pixels(u32, u32),
    Percentage(f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFilter {
    JpegCompression { quality: u8 },
    Brightness { percentage: u8 },
    Sharpen { strength: u8 },
    BoxBlur,
    GaussianBlur { sigma: f32 },
    Saturate { percentage: u16 },
    Noise { strength: u8, seed: Option<u64> },
    Resize { size: ResizeOption },
    Invert,
}

impl ImageFilter {
    pub const DEFAULTS: &[ImageFilter] = &[
        Self::JpegCompression { quality: 80 },
        Self::Brightness { percentage: 100 },
        Self::Sharpen { strength: 50 },
        Self::BoxBlur,
        Self::GaussianBlur { sigma: 2. },
        Self::Saturate { percentage: 100 },
        Self::Noise {
            strength: 10,
            seed: None,
        },
        Self::Resize {
            size: ResizeOption::Percentage(1.0, 1.0),
        },
        Self::Invert,
    ];

    pub const NAMES: &[&str] = &[
        "JPEG Compression",
        "Brightness",
        "Sharpen",
        "Box Blur",
        "Gaussian Blur",
        "Saturate",
        "Noise",
        "Resize",
        "Invert",
    ];

    pub fn name(&self) -> &str {
        match self {
            Self::JpegCompression { .. } => Self::NAMES[0],
            Self::Brightness { .. } => Self::NAMES[1],
            Self::Sharpen { .. } => Self::NAMES[2],
            Self::BoxBlur => Self::NAMES[3],
            Self::GaussianBlur { .. } => Self::NAMES[4],
            Self::Saturate { .. } => Self::NAMES[5],
            Self::Noise { .. } => Self::NAMES[6],
            Self::Resize { .. } => Self::NAMES[7],
            Self::Invert => Self::NAMES[8],
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
            Self::Sharpen { strength } => {
                Slider::new(strength, 0..=200).text("Strength (%)").ui(ui);
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
            Self::Resize { size } => {
                ui.horizontal(|ui| {
                    if ui
                        .add(RadioButton::new(
                            matches!(size, ResizeOption::Pixels(_, _)),
                            "Pixels",
                        ))
                        .clicked()
                    {
                        *size = ResizeOption::Pixels(128, 128);
                    }
                    if ui
                        .add(RadioButton::new(
                            matches!(size, ResizeOption::Percentage(_, _)),
                            "Factor",
                        ))
                        .clicked()
                    {
                        *size = ResizeOption::Percentage(1.0, 1.0);
                    }
                });
                match size {
                    ResizeOption::Pixels(width, height) => {
                        ui.horizontal(|ui| {
                            DragValue::new(width).suffix("px").ui(ui);
                            ui.label("Width");
                        });
                        ui.horizontal(|ui| {
                            DragValue::new(height).suffix("px").ui(ui);
                            ui.label("Height");
                        });
                    }
                    ResizeOption::Percentage(width, height) => {
                        ui.horizontal(|ui| {
                            DragValue::new(width).speed(0.01).range(0.0..=10.0).ui(ui);
                            ui.label("Width factor");
                        });
                        ui.horizontal(|ui| {
                            DragValue::new(height).speed(0.01).range(0.0..=10.0).ui(ui);
                            ui.label("Height factor");
                        });
                    }
                }
            }
            Self::BoxBlur | Self::Invert => {}
        }
    }

    // TODO: error management
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
            Self::Sharpen { strength } => {
                let strength = *strength as f32 / 100.;
                let edge = -1. * strength;
                let center = 4. * strength + 1.;
                img.filter3x3(&[0., edge, 0., edge, center, edge, 0., edge, 0.])
            }
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
            Self::Resize { size } => {
                let (width, height) = match size {
                    ResizeOption::Pixels(w, h) => (*w, *h),
                    ResizeOption::Percentage(w, h) => {
                        let w = (img.width() as f32 * w) as u32;
                        let h = (img.height() as f32 * h) as u32;
                        (w, h)
                    }
                };
                img.resize_exact(width, height, image::imageops::FilterType::Nearest)
            }
            Self::Invert => {
                let mut img = img;
                img.invert();
                img
            }
        }
    }
}
