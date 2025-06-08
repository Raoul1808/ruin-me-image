use eframe::egui::{Button, ComboBox, Ui};
use filter::ImageFilter;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

mod filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FilterCommand {
    enabled: bool,
    filter: ImageFilter,
}

#[derive(Debug, Default, Clone)]
pub struct CommandQueue {
    selected_filter: usize,
    queue: Vec<FilterCommand>,
}

impl CommandQueue {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_label("Add Filter")
                .selected_text(ImageFilter::NAMES[self.selected_filter])
                .show_index(
                    ui,
                    &mut self.selected_filter,
                    ImageFilter::NAMES.len(),
                    |i| ImageFilter::NAMES[i],
                );
            if ui.button("Add").clicked() {
                self.queue.push(FilterCommand {
                    enabled: true,
                    filter: ImageFilter::DEFAULTS[self.selected_filter].clone(),
                });
            }
        });
        ui.separator();
        let mut delete = vec![];
        let mut to_swap = None;
        let len = self.queue.len();
        for (i, filter) in self.queue.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut filter.enabled, "");
                let top = i == 0;
                if ui.add_enabled(!top, Button::new("⬆")).clicked() {
                    to_swap = Some((i, i - 1));
                }
                if ui.button("🗑").clicked() {
                    delete.push(i);
                }
                let bottom = i >= len - 1;
                if ui.add_enabled(!bottom, Button::new("⬇")).clicked() {
                    to_swap = Some((i, i + 1));
                }
                ui.label(format!("{}. {}", i + 1, filter.filter.name()))
            });
            ui.indent("wawa", |ui| {
                ui.add_enabled_ui(filter.enabled, |ui| filter.filter.ui(ui));
            });
        }
        if let Some((i1, i2)) = to_swap {
            let range = 0..self.queue.len();
            if i1 != i2 && range.contains(&i1) && range.contains(&i2) {
                self.queue.swap(i1, i2);
            }
        }
        for i in delete.into_iter().rev() {
            self.queue.remove(i);
        }
        if self.queue.is_empty() {
            ui.small("There is nothing here.");
        }
    }

    pub fn execute_clear(&mut self, img: DynamicImage) -> DynamicImage {
        let mut img = img;
        for filter in self.queue.drain(..) {
            if filter.enabled {
                img = filter.filter.apply(img);
            }
        }
        img
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn serialize(&self) -> ron::Result<String> {
        ron::to_string(&self.queue)
    }

    pub fn deserialize(&mut self, str: &str) -> ron::Result<()> {
        self.queue = ron::from_str(str)?;
        Ok(())
    }
}
