use dust_lang::{interpret, Result, Value};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct App {
    source: String,
    output: Result<Value>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, source: String) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        cc.egui_ctx.set_zoom_factor(1.5);

        let app = App {
            source,
            output: Ok(Value::default()),
        };

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or(app);
        } else {
            app
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
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.code_editor(&mut self.source);

            if ui.button("run").clicked() {
                self.output = interpret(&self.source);
            }

            ui.separator();

            let output_text = match &self.output {
                Ok(value) => value.to_string(),
                Err(error) => error.to_string(),
            };

            ui.label(output_text);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.hyperlink_to("source code", "https://git.jeffa.io/jeff/dust");
            });
        });
    }
}
