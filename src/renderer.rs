use image::{
    DynamicImage, Rgba, RgbaImage,
    imageops::{self, FilterType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Midnight,
    Aurora,
    Sunset,
    Mono,
}

impl Preset {
    pub const ALL: [Preset; 4] = [
        Preset::Midnight,
        Preset::Aurora,
        Preset::Sunset,
        Preset::Mono,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Preset::Midnight => "Midnight",
            Preset::Aurora => "Aurora",
            Preset::Sunset => "Sunset",
            Preset::Mono => "Mono",
        }
    }

    fn palette(self) -> ([u8; 3], [u8; 3], [u8; 3]) {
        match self {
            Preset::Midnight => ([7, 11, 26], [24, 31, 57], [95, 112, 255]),
            Preset::Aurora => ([9, 24, 35], [18, 62, 72], [102, 255, 207]),
            Preset::Sunset => ([39, 15, 34], [91, 36, 52], [255, 151, 86]),
            Preset::Mono => ([12, 12, 14], [45, 45, 49], [210, 210, 220]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelegramBackdropPalette {
    pub center: [u8; 3],
    pub edge: [u8; 3],
    pub symbol: [u8; 3],
    pub text: [u8; 3],
}

pub fn render_wallpaper(
    model: &DynamicImage,
    width: u32,
    height: u32,
    preset: Preset,
    telegram_backdrop: Option<TelegramBackdropPalette>,
    model_scale: f32,
    model_y: f32,
) -> RgbaImage {
    let width = width.max(1);
    let height = height.max(1);

    let mut out = if let Some(backdrop) = telegram_backdrop {
        render_telegram_backdrop(width, height, backdrop)
    } else {
        render_preset_background(width, height, preset)
    };

    let source = model.to_rgba8();

    let desired_side = width.min(height) as f32 * model_scale.clamp(0.20, 1.35);
    let fit_scale = (desired_side / source.width().max(source.height()).max(1) as f32).max(0.01);

    let target_w = (source.width() as f32 * fit_scale).round().max(1.0) as u32;
    let target_h = (source.height() as f32 * fit_scale).round().max(1.0) as u32;

    let resized = imageops::resize(&source, target_w, target_h, FilterType::Lanczos3);

    let x = (width as i64 - target_w as i64) / 2;
    let center_y = height as f32 * model_y.clamp(0.10, 0.90);
    let y = (center_y - target_h as f32 * 0.5).round() as i64;

    imageops::overlay(&mut out, &resized, x, y);

    out
}

fn render_telegram_backdrop(
    width: u32,
    height: u32,
    palette: TelegramBackdropPalette,
) -> RgbaImage {
    let mut out = RgbaImage::new(width, height);

    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.42;
    let max_dx = width as f32 * 0.62;
    let max_dy = height as f32 * 0.72;

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - center_x) / max_dx.max(1.0);
            let dy = (y as f32 - center_y) / max_dy.max(1.0);
            let distance = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
            let edge_mix = distance.powf(0.82);

            let mut rgb = [
                lerp(palette.center[0], palette.edge[0], edge_mix),
                lerp(palette.center[1], palette.edge[1], edge_mix),
                lerp(palette.center[2], palette.edge[2], edge_mix),
            ];

            // Telegram exposes a symbol color for the backdrop pattern. We do not
            // render the symbol asset yet, but a very soft central tint keeps the
            // palette visually faithful without inventing a fake pattern.
            let symbol_mix = ((1.0 - distance) * 0.055).clamp(0.0, 0.055);
            for (channel, value) in rgb.iter_mut().enumerate() {
                *value = blend(*value, palette.symbol[channel], symbol_mix);
            }

            out.put_pixel(x, y, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }
    }

    out
}

fn render_preset_background(width: u32, height: u32, preset: Preset) -> RgbaImage {
    let (top, bottom, glow) = preset.palette();
    let mut out = RgbaImage::new(width, height);

    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.38;
    let radius = width.min(height) as f32 * 0.55;

    for y in 0..height {
        let vertical = y as f32 / height as f32;

        for x in 0..width {
            let mut rgb = [
                lerp(top[0], bottom[0], vertical),
                lerp(top[1], bottom[1], vertical),
                lerp(top[2], bottom[2], vertical),
            ];

            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let distance = (dx * dx + dy * dy).sqrt();
            let intensity = (1.0 - distance / radius.max(1.0)).clamp(0.0, 1.0);
            let intensity = intensity * intensity * 0.48;

            for channel in 0..3 {
                rgb[channel] = blend(rgb[channel], glow[channel], intensity);
            }

            let nx = (x as f32 / width as f32 - 0.5).abs() * 2.0;
            let ny = (y as f32 / height as f32 - 0.5).abs() * 2.0;
            let vignette = (nx.max(ny).powf(2.2) * 0.28).clamp(0.0, 0.42);

            for channel in &mut rgb {
                *channel = ((*channel as f32) * (1.0 - vignette)) as u8;
            }

            out.put_pixel(x, y, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }
    }

    out
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

fn blend(a: u8, b: u8, t: f32) -> u8 {
    lerp(a, b, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_expected_size() {
        let model = DynamicImage::new_rgba8(128, 128);
        let result = render_wallpaper(&model, 1920, 1080, Preset::Midnight, None, 0.72, 0.50);
        assert_eq!(result.dimensions(), (1920, 1080));
    }

    #[test]
    fn renders_telegram_backdrop() {
        let model = DynamicImage::new_rgba8(64, 64);
        let palette = TelegramBackdropPalette {
            center: [100, 180, 240],
            edge: [20, 60, 120],
            symbol: [130, 210, 255],
            text: [255, 255, 255],
        };

        let result = render_wallpaper(
            &model,
            360,
            640,
            Preset::Midnight,
            Some(palette),
            0.72,
            0.50,
        );

        assert_eq!(result.dimensions(), (360, 640));
    }
}
