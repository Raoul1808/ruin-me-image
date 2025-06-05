use app::Application;
use eframe::{
    NativeOptions,
    egui::{ViewportBuilder, vec2},
};

mod app;

fn main() -> eframe::Result {
    let native_options = NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(vec2(1280., 720.)),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "Ruin me image",
        native_options,
        Box::new(|_cc| Ok(Box::new(Application::default()))),
    )
}
