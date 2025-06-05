use std::path::PathBuf;

use eframe::{
    App,
    egui::{
        CentralPanel, ColorImage, Image, TextureHandle, TextureOptions, Widget, load::SizedTexture,
        vec2,
    },
};
use image::{DynamicImage, EncodableLayout};

#[derive(Default)]
pub struct Application {
    path: Option<PathBuf>,
    img_tex: Option<(SizedTexture, TextureHandle)>,
}

impl App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, image!");
            ui.horizontal(|ui| {
                if ui.button("Browse").clicked() {
                    let file = rfd::FileDialog::new().set_title("Select image").pick_file();
                    if let Some(file) = file {
                        match image::open(&file) {
                            Ok(img) => {
                                let img = match &img {
                                    DynamicImage::ImageRgb8(image) => ColorImage::from_rgb(
                                        [image.width() as usize, image.height() as usize],
                                        image.as_bytes(),
                                    ),
                                    other => {
                                        let image = other.to_rgba8();
                                        ColorImage::from_rgba_unmultiplied(
                                            [image.width() as usize, image.height() as usize],
                                            image.as_bytes(),
                                        )
                                    }
                                };
                                let img_size = vec2(img.size[0] as f32, img.size[1] as f32);
                                let handle =
                                    ctx.load_texture("preview", img, TextureOptions::default());
                                let sized_image = SizedTexture::new(handle.id(), img_size);
                                self.img_tex = Some((sized_image, handle));
                                self.path = Some(file);
                            }
                            Err(e) => {
                                rfd::MessageDialog::new()
                                    .set_title("Error loading image")
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_description(format!(
                                        "Error while loading {}: {}",
                                        file.display(),
                                        e
                                    ))
                                    .show();
                            }
                        }
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
            if let Some((tex, _)) = self.img_tex.as_ref() {
                ui.separator();
                ui.label("Image preview");
                Image::new(*tex).shrink_to_fit().ui(ui);
            }
        });
    }
}
