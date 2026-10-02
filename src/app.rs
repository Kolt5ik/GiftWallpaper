use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use eframe::egui;
use image::DynamicImage;

use crate::{
    api::{GiftChangesClient, GiftDetails},
    renderer::{render_wallpaper, Preset},
};

enum WorkerMessage {
    Gifts(Result<Vec<String>, String>),
    Gift(Result<GiftDetails, String>),
    ModelImage {
        gift: String,
        model: String,
        result: Result<Vec<u8>, String>,
    },
}

pub struct GiftWallpaperApp {
    tx: Sender<WorkerMessage>,
    rx: Receiver<WorkerMessage>,

    gifts: Vec<String>,
    gift_search: String,
    model_search: String,
    selected_gift: Option<String>,
    details: Option<GiftDetails>,
    selected_model: Option<String>,

    model_image: Option<DynamicImage>,
    preview_texture: Option<egui::TextureHandle>,
    preview_size: [u32; 2],
    preset: Preset,

    export_width: u32,
    export_height: u32,
    model_scale: f32,
    model_y: f32,

    loading_gifts: bool,
    loading_details: bool,
    loading_image: bool,
    status: String,
    show_about: bool,
}

impl GiftWallpaperApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&cc.egui_ctx);

        let (tx, rx) = mpsc::channel();

        let mut app = Self {
            tx,
            rx,

            gifts: Vec::new(),
            gift_search: String::new(),
            model_search: String::new(),
            selected_gift: None,
            details: None,
            selected_model: None,

            model_image: None,
            preview_texture: None,
            preview_size: [360, 640],
            preset: Preset::Midnight,

            export_width: 1080,
            export_height: 1920,
            model_scale: 0.72,
            model_y: 0.50,

            loading_gifts: false,
            loading_details: false,
            loading_image: false,
            status: "Loading gifts…".to_owned(),
            show_about: false,
        };

        app.load_gifts();
        app
    }

    fn load_gifts(&mut self) {
        if self.loading_gifts {
            return;
        }

        self.loading_gifts = true;
        self.status = "Loading gift list…".to_owned();
        let tx = self.tx.clone();

        thread::spawn(move || {
            let result = GiftChangesClient::new().and_then(|api| api.list_gifts());
            let _ = tx.send(WorkerMessage::Gifts(result));
        });
    }

    fn load_gift_details(&mut self, gift: String) {
        self.loading_details = true;
        self.loading_image = false;
        self.status = format!("Loading {gift}…");
        self.details = None;
        self.selected_model = None;
        self.model_search.clear();
        self.model_image = None;
        self.preview_texture = None;

        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = GiftChangesClient::new().and_then(|api| api.gift_details(&gift));
            let _ = tx.send(WorkerMessage::Gift(result));
        });
    }

    fn load_model_image(&mut self, gift: String, model: String) {
        self.loading_image = true;
        self.status = format!("Loading {model}…");
        self.model_image = None;
        self.preview_texture = None;

        let tx = self.tx.clone();

        thread::spawn(move || {
            let result =
                GiftChangesClient::new().and_then(|api| api.model_png(&gift, &model, 1024));

            let _ = tx.send(WorkerMessage::ModelImage {
                gift,
                model,
                result,
            });
        });
    }

    fn process_worker_messages(&mut self, ctx: &egui::Context) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                WorkerMessage::Gifts(result) => {
                    self.loading_gifts = false;

                    match result {
                        Ok(gifts) => {
                            self.status = format!("{} gifts available", gifts.len());
                            self.gifts = gifts;
                        }
                        Err(error) => {
                            self.status = format!("Error: {error}");
                        }
                    }
                }

                WorkerMessage::Gift(result) => {
                    self.loading_details = false;

                    match result {
                        Ok(details) => {
                            let model_count = details.models.len();
                            self.status =
                                format!("{} loaded · {} models", details.name, model_count);
                            self.selected_gift = Some(details.name.clone());
                            self.details = Some(details);
                        }
                        Err(error) => {
                            self.status = format!("Error: {error}");
                        }
                    }
                }

                WorkerMessage::ModelImage {
                    gift,
                    model,
                    result,
                } => {
                    self.loading_image = false;

                    if self.selected_gift.as_deref() != Some(gift.as_str())
                        || self.selected_model.as_deref() != Some(model.as_str())
                    {
                        continue;
                    }

                    match result {
                        Ok(bytes) => match image::load_from_memory(&bytes) {
                            Ok(image) => {
                                self.model_image = Some(image);
                                self.rebuild_preview(ctx);
                                self.status = format!("{model} ready");
                            }
                            Err(error) => {
                                self.status = format!("Image decode error: {error}");
                            }
                        },
                        Err(error) => {
                            self.status = format!("Error: {error}");
                        }
                    }
                }
            }

            ctx.request_repaint();
        }
    }

    fn rebuild_preview(&mut self, ctx: &egui::Context) {
        let Some(model) = self.model_image.as_ref() else {
            self.preview_texture = None;
            return;
        };

        let [preview_w, preview_h] =
            preview_dimensions(self.export_width, self.export_height, 640);

        let wallpaper = render_wallpaper(
            model,
            preview_w,
            preview_h,
            self.preset,
            self.model_scale,
            self.model_y,
        );

        self.preview_size = [wallpaper.width(), wallpaper.height()];

        let size = [wallpaper.width() as usize, wallpaper.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, wallpaper.as_raw());

        self.preview_texture = Some(ctx.load_texture(
            "wallpaper-preview",
            color_image,
            egui::TextureOptions::LINEAR,
        ));
    }

    fn export_wallpaper(&mut self) {
        let Some(model) = self.model_image.as_ref() else {
            self.status = "Choose a model first".to_owned();
            return;
        };

        let image = render_wallpaper(
            model,
            self.export_width,
            self.export_height,
            self.preset,
            self.model_scale,
            self.model_y,
        );

        let gift = self.selected_gift.as_deref().unwrap_or("gift");
        let model_name = self.selected_model.as_deref().unwrap_or("model");

        let file_name = format!(
            "{}-{}-{}x{}.png",
            slug(gift),
            slug(model_name),
            self.export_width,
            self.export_height
        );

        let export_dir = PathBuf::from("exports");
        if let Err(error) = std::fs::create_dir_all(&export_dir) {
            self.status = format!("Could not create exports folder: {error}");
            return;
        }

        let path = export_dir.join(file_name);

        match image.save(&path) {
            Ok(()) => {
                self.status = format!("Saved to {}", path.display());
            }
            Err(error) => {
                self.status = format!("Export failed: {error}");
            }
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("GiftWallpaper");
        ui.label(
            egui::RichText::new("Telegram Gift wallpaper studio")
                .weak()
                .size(13.0),
        );
        ui.add_space(12.0);

        ui.label("Gift");
        ui.text_edit_singleline(&mut self.gift_search);

        if self.loading_gifts {
            ui.spinner();
        }

        let search = self.gift_search.trim().to_lowercase();
        let matching: Vec<String> = self
            .gifts
            .iter()
            .filter(|gift| search.is_empty() || gift.to_lowercase().contains(&search))
            .cloned()
            .collect();

        egui::ScrollArea::vertical()
            .id_salt("gift-list")
            .max_height(155.0)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                for gift in matching {
                    let selected = self.selected_gift.as_deref() == Some(gift.as_str());
                    if ui.selectable_label(selected, &gift).clicked() {
                        self.selected_gift = Some(gift.clone());
                        self.load_gift_details(gift);
                    }
                }
            });

        ui.separator();

        ui.label("Model");
        ui.text_edit_singleline(&mut self.model_search);

        if self.loading_details {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading models…");
            });
        }

        let model_search = self.model_search.trim().to_lowercase();
        let models = self
            .details
            .as_ref()
            .map(|details| {
                details
                    .models
                    .iter()
                    .filter(|model| {
                        model_search.is_empty()
                            || model.name.to_lowercase().contains(&model_search)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        egui::ScrollArea::vertical()
            .id_salt("model-list")
            .max_height(220.0)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                for model in models {
                    let selected = self.selected_model.as_deref() == Some(model.name.as_str());

                    let label = if let Some(rarity) = model.rarity {
                        format!("{}  ·  {}", model.name, format_rarity(rarity))
                    } else {
                        model.name.clone()
                    };

                    if ui.selectable_label(selected, label).clicked() {
                        self.selected_model = Some(model.name.clone());

                        if let Some(gift) = self.selected_gift.clone() {
                            self.load_model_image(gift, model.name);
                        }
                    }
                }
            });

        ui.separator();

        ui.label("Style");
        let mut preview_changed = false;

        egui::ComboBox::from_id_salt("preset")
            .selected_text(self.preset.label())
            .show_ui(ui, |ui| {
                for preset in Preset::ALL {
                    preview_changed |= ui
                        .selectable_value(&mut self.preset, preset, preset.label())
                        .changed();
                }
            });

        ui.add_space(8.0);
        ui.label("Gift size");
        preview_changed |= ui
            .add(
                egui::Slider::new(&mut self.model_scale, 0.25..=1.30)
                    .text("scale")
                    .show_value(true),
            )
            .changed();

        ui.label("Vertical position");
        preview_changed |= ui
            .add(
                egui::Slider::new(&mut self.model_y, 0.15..=0.85)
                    .text("Y")
                    .show_value(true),
            )
            .changed();

        ui.separator();
        ui.label("Canvas size");

        let old_size = (self.export_width, self.export_height);

        egui::ComboBox::from_id_salt("resolution")
            .selected_text(format!("{} × {}", self.export_width, self.export_height))
            .show_ui(ui, |ui| {
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 1080, 1920, "Phone 9:16");
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 1440, 2560, "Phone QHD");
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 2160, 3840, "Phone 4K");
                ui.separator();
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 1920, 1080, "Desktop FHD");
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 2560, 1440, "Desktop QHD");
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 3840, 2160, "Desktop 4K");
                ui.separator();
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 1080, 1080, "Square");
                resolution_option(ui, &mut self.export_width, &mut self.export_height, 1920, 1200, "Desktop 16:10");
            });

        ui.horizontal(|ui| {
            ui.label("W");
            ui.add(
                egui::DragValue::new(&mut self.export_width)
                    .range(256..=7680)
                    .speed(8.0),
            );
            ui.label("H");
            ui.add(
                egui::DragValue::new(&mut self.export_height)
                    .range(256..=7680)
                    .speed(8.0),
            );
        });

        ui.label(
            egui::RichText::new("Можно вписать любой размер вручную")
                .weak()
                .small(),
        );

        if old_size != (self.export_width, self.export_height) {
            preview_changed = true;
        }

        if preview_changed {
            self.rebuild_preview(ctx);
        }

        ui.add_space(12.0);

        let can_export = self.model_image.is_some() && !self.loading_image;
        if ui
            .add_enabled(
                can_export,
                egui::Button::new("Export PNG")
                    .min_size(egui::vec2(ui.available_width(), 36.0)),
            )
            .clicked()
        {
            self.export_wallpaper();
        }

        ui.add_space(6.0);

        if ui.button("About / Credits").clicked() {
            self.show_about = true;
        }
    }

    fn preview(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(self.selected_gift.as_deref().unwrap_or("Choose a gift"));

            if let Some(details) = &self.details {
                if let Some(id) = &details.id {
                    ui.label(
                        egui::RichText::new(format!("Gift ID: {id}"))
                            .weak()
                            .small(),
                    );
                }
            }

            ui.label(
                egui::RichText::new(format!(
                    "{} × {}",
                    self.export_width, self.export_height
                ))
                .weak(),
            );

            ui.add_space(10.0);

            if self.loading_image {
                ui.add_space(190.0);
                ui.spinner();
                ui.label("Downloading 1024px model…");
                return;
            }

            if let Some(texture) = &self.preview_texture {
                let available = ui.available_size();
                let source_w = self.preview_size[0].max(1) as f32;
                let source_h = self.preview_size[1].max(1) as f32;

                let max_w = (available.x - 32.0).max(200.0);
                let max_h = (available.y - 72.0).max(200.0);
                let fit = (max_w / source_w).min(max_h / source_h).min(1.0);

                let size = egui::vec2(source_w * fit, source_h * fit);

                egui::Frame::new()
                    .corner_radius(egui::CornerRadius::same(18))
                    .shadow(egui::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(90),
                    })
                    .show(ui, |ui| {
                        ui.add(egui::Image::new((texture.id(), size)));
                    });

                if let Some(model) = &self.selected_model {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(model).strong());
                }
            } else {
                ui.add_space(150.0);
                ui.label(
                    egui::RichText::new("Select a gift and a model to generate a wallpaper")
                        .weak()
                        .size(16.0),
                );
            }
        });
    }



    fn footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&self.status).weak().small());

            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.hyperlink_to(
                        "Powered by @GiftChanges · api.changes.tg",
                        "https://api.changes.tg/",
                    );
                },
            );
        });

        ui.add_space(2.0);

        ui.horizontal_centered(|ui| {
            ui.label(
                egui::RichText::new("Open Source")
                    .small()
                    .strong()
                    .color(egui::Color32::from_rgb(180, 185, 195)),
            );

            ui.separator();

            ui.hyperlink_to(
                egui::RichText::new("GitHub · Kolt5ik")
                    .small()
                    .strong()
                    .color(egui::Color32::from_rgb(210, 214, 222)),
                "https://github.com/Kolt5ik",
            );
        });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }

        egui::Window::new("About GiftWallpaper")
            .open(&mut self.show_about)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("GiftWallpaper");
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(8.0);
                ui.label("Open-source desktop wallpaper generator for Telegram collectible gifts.");
                ui.hyperlink_to("Project repository: github.com/Kolt5ik", "https://github.com/Kolt5ik");
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Powered by @GiftChanges (api.changes.tg)").strong(),
                );
                ui.hyperlink_to("Open Gift Changes API", "https://api.changes.tg/");
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "Gift metadata and gift assets are provided by the Gift Changes API.",
                    )
                    .weak(),
                );
            });
    }
}

impl eframe::App for GiftWallpaperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.process_worker_messages(&ctx);

        egui::Panel::bottom("footer")
            .exact_size(56.0)
            .show(ui, |ui| {
                self.footer(ui);
            });

        egui::Panel::left("sidebar")
            .default_size(330.0)
            .min_size(300.0)
            .max_size(430.0)
            .resizable(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("sidebar-scroll")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        self.sidebar(&ctx, ui);
                    });
            });

        egui::CentralPanel::default_margins().show(ui, |ui| {
            self.preview(ui);
        });

        self.about_window(&ctx);

        if self.loading_gifts || self.loading_details || self.loading_image {
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
        }
    }
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(16, 18, 24);
    visuals.window_fill = egui::Color32::from_rgb(20, 23, 31);
    visuals.extreme_bg_color = egui::Color32::from_rgb(11, 13, 18);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);

    // Make scrollbars easier to grab with the mouse.
    visuals.widgets.inactive.bg_stroke.width = 1.0;
    ctx.set_visuals(visuals);
}

fn format_rarity(value: f64) -> String {
    if value < 0.1 {
        format!("{value:.3}%")
    } else if value < 1.0 {
        format!("{value:.2}%")
    } else {
        format!("{value:.1}%")
    }
}

fn resolution_option(
    ui: &mut egui::Ui,
    width: &mut u32,
    height: &mut u32,
    option_width: u32,
    option_height: u32,
    label: &str,
) {
    let selected = *width == option_width && *height == option_height;
    if ui
        .selectable_label(
            selected,
            format!("{label} — {option_width} × {option_height}"),
        )
        .clicked()
    {
        *width = option_width;
        *height = option_height;
    }
}

fn preview_dimensions(width: u32, height: u32, max_side: u32) -> [u32; 2] {
    let width = width.max(1);
    let height = height.max(1);

    if width >= height {
        let preview_w = max_side;
        let preview_h = ((height as f64 / width as f64) * max_side as f64)
            .round()
            .clamp(120.0, max_side as f64) as u32;
        [preview_w, preview_h]
    } else {
        let preview_h = max_side;
        let preview_w = ((width as f64 / height as f64) * max_side as f64)
            .round()
            .clamp(120.0, max_side as f64) as u32;
        [preview_w, preview_h]
    }
}

fn slug(value: &str) -> String {
    let mut output = String::new();
    let mut previous_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash {
            output.push('-');
            previous_dash = true;
        }
    }

    output.trim_matches('-').to_owned()
}
