#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod renderer;

use app::GiftWallpaperApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([900.0, 620.0])
            .with_title("GiftWallpaper"),
        ..Default::default()
    };

    eframe::run_native(
        "GiftWallpaper",
        options,
        Box::new(|cc| Ok(Box::new(GiftWallpaperApp::new(cc)))),
    )
}
