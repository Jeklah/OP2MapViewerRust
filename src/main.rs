//! Main entry point for OP2MapViewer

use eframe::egui;

mod map {
    pub mod types;
    pub mod loader;
}

mod ui {
    pub mod app;
    pub mod map_view;
    pub mod cell_info;
}

use ui::app::MapViewerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([640.0, 480.0])
            .with_resizable(true)
            .with_title("OP2MapViewer"),
        ..Default::default()
    };

    eframe::run_native(
        "OP2MapViewer",
        options,
        Box::new(|cc| Box::new(MapViewerApp::new(cc))),
    )
}
