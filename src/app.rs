use std::path::PathBuf;

use eframe::{
    App,
    egui::{
        self, Button, CentralPanel, ColorImage, Image, SidePanel, TextureHandle, TextureOptions,
        Ui, Widget, load::SizedTexture,
    },
};
use image::{DynamicImage, EncodableLayout};

use crate::{commands::CommandQueue, worker::ImageWorker};

pub struct Application {
    worker: ImageWorker,
    path: Option<PathBuf>,
    base_img: Option<DynamicImage>,
    img: ImageLoadState,
    queue: CommandQueue,
}

impl Application {
    pub fn new() -> Self {
        Self {
            worker: ImageWorker::new(),
            path: None,
            base_img: None,
            img: ImageLoadState::None,
            queue: CommandQueue::default(),
        }
    }

    fn update_image_state(&mut self, ctx: &egui::Context) {
        if let Some(res) = self.worker.try_recv() {
            match res {
                Ok(img) => {
                    if matches!(self.img, ImageLoadState::Loading) {
                        self.base_img = Some(img.clone());
                    }
                    let img_gui = match img.clone() {
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
                    let handle = ctx.load_texture("preview", img_gui, TextureOptions::default());
                    let tex = SizedTexture::from_handle(&handle);
                    self.img = ImageLoadState::Loaded { handle, tex, img }
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

    fn show_controls(&mut self, ui: &mut Ui) {
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
        if self.path.is_some() {
            self.queue.ui(ui);
            let mut render_request = false;
            match &self.img {
                ImageLoadState::None | ImageLoadState::Loading | ImageLoadState::Rendering => {
                    ui.add_enabled(false, Button::new("Render"));
                    ui.add_enabled(false, Button::new("Save current render"));
                }
                ImageLoadState::Loaded { img, .. } => {
                    let base_img = self
                        .base_img
                        .as_ref()
                        .expect("base image should be loaded!!!");
                    ui.horizontal(|ui| {
                        if ui.button("Render").clicked() {
                            println!("Requesting image render with {} filters", self.queue.len());
                            self.worker
                                .request_render(self.queue.clone(), base_img.clone());
                            render_request = true;
                        }
                        if ui.button("Save current render").clicked() && !render_request {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select path to save image")
                                .save_file()
                            {
                                match img.save(path) {
                                    Ok(_) => {
                                        rfd::MessageDialog::new()
                                            .set_title("Image savec")
                                            .set_level(rfd::MessageLevel::Info)
                                            .set_description("Image saved successfully")
                                            .show();
                                    }
                                    Err(e) => {
                                        rfd::MessageDialog::new()
                                            .set_title("Image error")
                                            .set_level(rfd::MessageLevel::Error)
                                            .set_description(format!("Failed to save image: {}", e))
                                            .show();
                                    }
                                }
                            }
                        }
                    });
                }
            }
            if render_request {
                self.img = ImageLoadState::Rendering;
            }
        }
    }
}

enum ImageLoadState {
    None,
    Loading,
    Rendering,
    Loaded {
        #[allow(dead_code)]
        handle: TextureHandle,
        tex: SizedTexture,
        img: DynamicImage,
    },
}

impl App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.update_image_state(ctx);
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ruin me image");
            ui.separator();
            SidePanel::left("queue").show_inside(ui, |ui| self.show_controls(ui));
            CentralPanel::default().show_inside(ui, |ui| match &self.img {
                ImageLoadState::None => {
                    ui.label("Select an image to view");
                }
                ImageLoadState::Loading => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading image...");
                    });
                }
                ImageLoadState::Rendering => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Rendering image...");
                    });
                }
                ImageLoadState::Loaded { tex, .. } => {
                    ui.label("Image preview");
                    Image::new(*tex).shrink_to_fit().ui(ui);
                }
            });
        });
    }
}
