#![windows_subsystem = "windows"]

mod git;
mod state;
mod ui;
mod watcher;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn load_icon(bytes: &[u8]) -> egui::IconData {
    let img = image::load_from_memory(bytes)
        .expect("invalid icon PNG")
        .into_rgba8();
    let (w, h) = image::GenericImageView::dimensions(&img);
    egui::IconData { rgba: img.into_raw(), width: w, height: h }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("gitrove=info".parse()?))
        .init();

    info!("starting gitrove");

    let icon_dark  = std::sync::Arc::new(load_icon(include_bytes!("../assets/icon-dark.png")));
    let icon_light = std::sync::Arc::new(load_icon(include_bytes!("../assets/icon-light.png")));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("gitrove")
            .with_icon(icon_dark.clone())
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "gitrove",
        native_options,
        Box::new(move |cc| Ok(Box::new(ui::App::new(cc, icon_dark, icon_light)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}
