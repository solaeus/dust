#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;

mod app;

#[derive(Parser)]
struct Args {
    // Path to the file to read.
    path: PathBuf,
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let path = Args::parse().path;
    let source = read_to_string(&path).unwrap();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dust GUI",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc, source))),
    )
}
