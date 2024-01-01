use std::{fs::read_to_string, path::PathBuf};

use dust_lang::{Interpreter, Map, Result, Value};
use egui::{Align, Color32, Layout, RichText, ScrollArea};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct App {
    path: String,
    source: String,
    context: Map,
    #[serde(skip)]
    interpreter: Interpreter,
    output: Result<Value>,
    error: Option<String>,
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
                error: None,
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
            ui.columns(2, |columns| {
                ScrollArea::vertical()
                    .id_source("source")
                    .show(&mut columns[0], |ui| {
                        if let Some(error) = &self.error {
                            ui.label(RichText::new(error).color(Color32::LIGHT_RED));
                        }

                        ui.text_edit_singleline(&mut self.path);
                        ui.code_editor(&mut self.source);

                        if ui.button("read").clicked() {
                            match read_to_string(&self.path) {
                                Ok(source) => {
                                    self.source = source;
                                    self.error = None;
                                }
                                Err(error) => self.error = Some(error.to_string()),
                            }
                        }

                        if ui.button("run").clicked() {
                            self.output = self.interpreter.run(&self.source);
                        }
                    });
                ScrollArea::vertical()
                    .id_source("output")
                    .show(&mut columns[1], |ui| match &self.output {
                        Ok(value) => display_value(value, ui),
                        Err(error) => {
                            ui.label(RichText::new(error.to_string()).color(Color32::LIGHT_RED));

                            display_value(&Value::Map(self.context.clone()), ui);

                            match &self.output {
                                Ok(value) => {
                                    display_value(value, ui);
                                }
                                Err(error) => {
                                    ui.label(error.to_string());
                                }
                            }
                        }
                    });
            });
        });
    }
}

fn display_value(value: &Value, ui: &mut egui::Ui) {
    match value {
        Value::List(list) => {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::auto())
                .column(Column::auto());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("index");
                    });
                })
                .body(|mut body| {
                    for (index, value) in list.items().iter().enumerate() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(index.to_string());
                            });
                            row.col(|ui| {
                                display_value(value, ui);
                            });
                        });
                    }
                });
        }
        Value::Map(map) => {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::auto())
                .column(Column::auto());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("key");
                    });
                })
                .body(|mut body| {
                    for (key, (value, _)) in map.variables().unwrap().iter() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(key);
                            });
                            row.col(|ui| {
                                display_value(value, ui);
                            });
                        });
                    }
                });
        }
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
        Value::Option(option) => match option {
            Some(value) => {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .column(Column::auto());

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("some");
                        });
                    })
                    .body(|mut body| {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                display_value(value, ui);
                            });
                        });
                    });
            }
            None => {
                ui.label("none");
            }
        },
        Value::BuiltIn(_) => todo!(),
    }
}
