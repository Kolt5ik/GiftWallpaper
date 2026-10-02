#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod app;
mod renderer;

use app::GiftWallpaperApp;

fn load_app_icon() -> eframe::egui::IconData {
    let rgba = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("embedded app icon must be a valid image")
        .into_rgba8();

    eframe::egui::IconData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([900.0, 620.0])
            .with_title("GiftWallpaper")
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "GiftWallpaper",
        options,
        Box::new(|cc| Ok(Box::new(GiftWallpaperApp::new(cc)))),
    )
}
