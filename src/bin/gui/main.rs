#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use clap::Parser;

mod app;

#[derive(Parser)]
struct Args {
    // Path to the file to read.
    path: Option<PathBuf>,
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let path = if let Some(path) = Args::parse().path {
        path
    } else {
        PathBuf::new()
    };
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dust GUI",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc, path))),
    )
}
