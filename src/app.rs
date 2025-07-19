use std::{fs, path::PathBuf};

use eframe::{
    App,
    egui::{
        self, Button, CentralPanel, ColorImage, Image, ProgressBar, SidePanel, TextureHandle,
        TextureOptions, Ui, Widget, load::SizedTexture, vec2,
    },
};
use image::{DynamicImage, EncodableLayout};

use crate::{
    commands::CommandQueue,
    worker::{ImageWorker, WorkerResult},
};

pub struct Application {
    worker: ImageWorker,
    path: Option<PathBuf>,
    base_img: Option<DynamicImage>,
    img: ImageLoadState,
    queue: CommandQueue,
}

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

fn serialize_to_file(queue: &CommandQueue) -> BoxResult<()> {
    let path = rfd::FileDialog::new()
        .set_title("Select saving location")
        .save_file();
    let path = match path {
        Some(p) => p,
        None => return Ok(()),
    };
    let queue = queue.serialize()?;
    fs::write(path, queue)?;
    Ok(())
}

fn deserialize_from_file(queue: &mut CommandQueue) -> BoxResult<()> {
    let path = rfd::FileDialog::new()
        .set_title("Select queue file")
        .pick_file();
    let path = match path {
        Some(p) => p,
        None => return Ok(()),
    };
    let contents = fs::read_to_string(path)?;
    queue.deserialize(&contents)?;
    Ok(())
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
                WorkerResult::Finished(img) => {
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
                WorkerResult::Progress(i) => {
                    if let ImageLoadState::Rendering { progress, .. } = &mut self.img {
                        *progress = i;
                    }
                }
                WorkerResult::Error(e) => {
                    rfd::MessageDialog::new()
                        .set_title("Image error")
                        .set_level(rfd::MessageLevel::Error)
                        .set_description(format!("{e}"));
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
            ui.horizontal(|ui| {
                if ui.button("Load Queue").clicked() {
                    match deserialize_from_file(&mut self.queue) {
                        Ok(_) => {}
                        Err(e) => {
                            rfd::MessageDialog::new()
                                .set_title("Queue loading error")
                                .set_level(rfd::MessageLevel::Error)
                                .set_description(format!("Failed to load queue from file: {e}"))
                                .show();
                        }
                    }
                }
                if ui.button("Export Queue").clicked() {
                    match serialize_to_file(&self.queue) {
                        Ok(_) => {}
                        Err(e) => {
                            rfd::MessageDialog::new()
                                .set_title("Queue saving error")
                                .set_level(rfd::MessageLevel::Error)
                                .set_description(format!("Failed to save queue to file: {e}"))
                                .show();
                        }
                    }
                }
            });
            ui.separator();
            let available_width = ui.available_width();
            let available_height = {
                let available = ui.available_height();
                let spacing = ui.spacing();
                available - spacing.item_spacing.y * 9. - spacing.interact_size.y
            };
            ui.allocate_ui(vec2(available_width, available_height), |ui| {
                self.queue.ui(ui);
                ui.add_space(ui.available_height());
            });
            ui.separator();
            let mut render_request = false;
            match &self.img {
                ImageLoadState::None
                | ImageLoadState::Loading
                | ImageLoadState::Rendering { .. } => {
                    ui.horizontal(|ui| {
                        ui.add_enabled(false, Button::new("Render"));
                        ui.add_enabled(false, Button::new("Save current render"));
                    });
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
                                            .set_description(format!("Failed to save image: {e}"))
                                            .show();
                                    }
                                }
                            }
                        }
                    });
                }
            }
            if render_request {
                self.img = ImageLoadState::Rendering {
                    progress: 0,
                    total: self.queue.len(),
                };
            }
        } else {
            let available_width = ui.available_width();
            let available_height = {
                let available = ui.available_height();
                let spacing = ui.spacing();
                available - spacing.item_spacing.y * 5.
            };
            ui.allocate_ui(vec2(available_width, available_height), |ui| {
                self.queue.ui(ui);
                ui.add_space(ui.available_height());
            });
        }
        ui.small(format!("ruin-me-image v{APP_VERSION}"));
    }
}

enum ImageLoadState {
    None,
    Loading,
    Rendering {
        progress: usize,
        total: usize,
    },
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
                ImageLoadState::Rendering { progress, total } => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Rendering image...");
                    });
                    let progress_percent = *progress as f32 / *total as f32;
                    ProgressBar::new(progress_percent)
                        .animate(true)
                        .text(format!("{progress} / {total} filters processed"))
                        .ui(ui);
                }
                ImageLoadState::Loaded { tex, .. } => {
                    ui.label("Image preview");
                    Image::new(*tex).shrink_to_fit().ui(ui);
                }
            });
        });
    }
}
