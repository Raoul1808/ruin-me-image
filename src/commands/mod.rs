use eframe::egui::{ComboBox, Id, Ui, collapsing_header::CollapsingState};
use filter::ImageFilter;
use image::DynamicImage;

mod filter;

#[derive(Debug, Clone)]
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
        for (i, filter) in self.queue.iter_mut().enumerate() {
            let id = Id::new(format!("Collapsing{}{}", filter.filter.name(), i));
            let state = CollapsingState::load_with_default_open(ui.ctx(), id, false);
            state
                .show_header(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut filter.enabled, "");
                        ui.label(format!("{}. {}", i + 1, filter.filter.name()))
                    });
                })
                .body(|ui| ui.add_enabled_ui(filter.enabled, |ui| filter.filter.ui(ui)));
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
}
