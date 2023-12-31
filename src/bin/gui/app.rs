use std::{fs::read_to_string, path::PathBuf};

use dust_lang::{Interpreter, Map, Result, Value};
use egui::{Align, Color32, Layout, RichText};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct App {
    path: String,
    source: String,
    context: Map,
    #[serde(skip)]
    interpreter: Interpreter,
    output: Result<Value>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, path: PathBuf) -> Self {
        fn create_app(path: PathBuf) -> App {
            let context = Map::new();
            let mut interpreter = Interpreter::new(context.clone());
            let read_source = read_to_string(&path);
            let source = if let Ok(source) = read_source {
                source
            } else {
                String::new()
            };
            let output = interpreter.run(&source);

            App {
                path: path.to_string_lossy().to_string(),
                source,
                context,
                interpreter,
                output,
            }
        }

        cc.egui_ctx.set_zoom_factor(1.2);

        if path.is_file() {
            create_app(path)
        } else {
            if let Some(storage) = cc.storage {
                return eframe::get_value(storage, eframe::APP_KEY)
                    .unwrap_or_else(|| create_app(path));
            } else {
                create_app(path)
            }
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(16.0);

                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.hyperlink_to("source code", "https://git.jeffa.io/jeff/dust");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.with_layout(Layout::top_down(Align::Min).with_main_justify(true), |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.text_edit_singleline(&mut self.path);

                        if ui.button("read").clicked() {
                            self.source = read_to_string(&self.path).unwrap();
                        }

                        if ui.button("run").clicked() {
                            self.output = self.interpreter.run(&self.source);
                        }
                    });
                    ui.code_editor(&mut self.source);
                });

                match &self.output {
                    Ok(value) => {
                        display_value(value, ui);
                    }
                    Err(error) => {
                        ui.label(error.to_string());
                    }
                }
            });
        });
    }
}

fn display_value(value: &Value, ui: &mut egui::Ui) {
    match value {
        Value::List(list) => {
            ui.collapsing("list", |ui| {
                for value in list.items().iter() {
                    display_value(value, ui);
                }
            });
        }
        Value::Map(_) => todo!(),
        Value::Function(function) => {
            ui.label(function.to_string());
        }
        Value::String(string) => {
            ui.label(RichText::new(string).color(Color32::GREEN));
        }
        Value::Float(float) => {
            ui.label(float.to_string());
        }
        Value::Integer(integer) => {
            ui.label(RichText::new(integer.to_string()).color(Color32::BLUE));
        }
        Value::Boolean(boolean) => {
            ui.label(RichText::new(boolean.to_string()).color(Color32::RED));
        }
        Value::Option(_) => todo!(),
    }
}
