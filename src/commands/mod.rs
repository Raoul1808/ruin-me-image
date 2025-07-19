use eframe::egui::{Align, Button, ComboBox, Layout, ScrollArea, Ui, style::ScrollStyle};
use filter::ImageFilter;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

mod filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCommand {
    enabled: bool,
    filter: ImageFilter,
}

impl FilterCommand {
    pub fn execute(self, img: DynamicImage) -> DynamicImage {
        if self.enabled {
            self.filter.apply(img)
        } else {
            img
        }
    }
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
        ui.scope(|ui| {
            ui.spacing_mut().scroll = ScrollStyle::solid();
            ScrollArea::vertical().show(ui, |ui| {
                let len = self.queue.len();
                for (i, filter) in self.queue.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut filter.enabled, "");
                        ui.label(format!("{}. {}", i + 1, filter.filter.name()));
                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            let bottom = i >= len - 1;
                            if ui.add_enabled(!bottom, Button::new("⬇")).clicked() {
                                to_swap = Some((i, i + 1));
                            }
                            if ui.button("🗑").clicked() {
                                delete.push(i);
                            }
                            let top = i == 0;
                            if ui.add_enabled(!top, Button::new("⬆")).clicked() {
                                to_swap = Some((i, i - 1));
                            }
                        });
                    });
                    ui.indent("wawa", |ui| {
                        ui.add_enabled_ui(filter.enabled, |ui| filter.filter.ui(ui));
                    });
                }
            });
        });
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

    pub fn into_iter(self) -> std::vec::IntoIter<FilterCommand> {
        self.queue.into_iter()
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
