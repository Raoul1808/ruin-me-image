use std::path::PathBuf;

use eframe::{
    App,
    egui::{
        self, CentralPanel, ColorImage, Image, TextureHandle, TextureOptions, Widget,
        load::SizedTexture,
    },
};
use image::{DynamicImage, EncodableLayout};

use crate::worker::ImageWorker;

pub struct Application {
    worker: ImageWorker,
    path: Option<PathBuf>,
    img: ImageLoadState,
}

impl Application {
    pub fn new() -> Self {
        Self {
            worker: ImageWorker::new(),
            path: None,
            img: ImageLoadState::None,
        }
    }

    fn update_image_state(&mut self, ctx: &egui::Context) {
        if let Some(res) = self.worker.try_recv() {
            match res {
                Ok(img) => {
                    let img = match img {
                        DynamicImage::ImageRgb8(img) => ColorImage::from_rgb(
                            [img.width() as usize, img.height() as usize],
                            img.as_bytes(),
                        ),
                        other => {
                            let img = other.to_rgba8();
                            ColorImage::from_rgba_unmultiplied(
                                [img.width() as usize, img.height() as usize],
                                img.as_bytes(),
                            )
                        }
                    };
                    let handle = ctx.load_texture("preview", img, TextureOptions::default());
                    let tex = SizedTexture::from_handle(&handle);
                    self.img = ImageLoadState::Loaded { handle, tex }
                }
                Err(e) => {
                    rfd::MessageDialog::new()
                        .set_title("Image error")
                        .set_level(rfd::MessageLevel::Error)
                        .set_description(format!("{}", e));
                }
            }
        }
    }
}

enum ImageLoadState {
    None,
    Loading,
    Loaded {
        #[allow(dead_code)]
        handle: TextureHandle,
        tex: SizedTexture,
    },
}

impl App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.update_image_state(ctx);
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, image!");
            ui.horizontal(|ui| {
                if ui.button("Browse").clicked() {
                    let file = rfd::FileDialog::new().set_title("Select image").pick_file();
                    if let Some(file) = file {
                        self.img = ImageLoadState::Loading;
                        self.worker.request_image_load(file.clone());
                        self.path = Some(file);
                    }
                }
                ui.label(format!(
                    "Selected: {}",
                    self.path
                        .as_ref()
                        .map(|d| d.display().to_string())
                        .unwrap_or("None".into())
                ))
            });
            ui.separator();
            match &self.img {
                ImageLoadState::None => {
                    ui.label("Select an image to view");
                }
                ImageLoadState::Loading => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading image...");
                    });
                }
                ImageLoadState::Loaded { tex, .. } => {
                    ui.label("Image preview");
                    Image::new(*tex).shrink_to_fit().ui(ui);
                }
            }
        });
    }
}
