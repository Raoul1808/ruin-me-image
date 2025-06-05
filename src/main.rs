use app::Application;
use eframe::NativeOptions;

mod app;

fn main() -> eframe::Result {
    eframe::run_native(
        "Ruin me image",
        NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(Application::default()))),
    )
}
