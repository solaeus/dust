use eframe::{
    egui::{
        plot::{Bar, BarChart, Line, Plot as EguiPlot, PlotPoints},
        CentralPanel, Context, Direction, Layout, RichText, ScrollArea, Ui,
    },
    emath::Align,
    epaint::{Color32, Stroke},
    run_native, NativeOptions,
};
use egui_extras::{Column, StripBuilder, TableBuilder};

use crate::{Error, Result, Table, Tool, ToolInfo, Value, ValueType, VariableMap};

pub struct BarGraph;

impl Tool for BarGraph {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "bar_graph",
            description: "Render a list of values as a bar graph.",
            group: "gui",
            inputs: vec![ValueType::ListOf(Box::new(ValueType::List))],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_list()?;
        let mut bars = Vec::new();

        for (index, value) in argument.iter().enumerate() {
            let list = value.as_list()?;
            let mut name = None;
            let mut height = None;

            for value in list {
                match value {
                    Value::Float(float) => {
                        if height.is_none() {
                            height = Some(float);
                        }
                    }
                    Value::Integer(_integer) => {}
                    Value::String(string) => name = Some(string),
                    Value::Boolean(_)
                    | Value::List(_)
                    | Value::Map(_)
                    | Value::Table(_)
                    | Value::Time(_)
                    | Value::Function(_)
                    | Value::Empty => continue,
                }
            }

            let height =
                match height {
                    Some(height) => *height,
                    None => return Err(Error::CustomMessage(
                        "Could not create bar graph. No float value was found to use as a height."
                            .to_string(),
                    )),
                };

            let bar = Bar::new(index as f64, height).name(name.unwrap_or(&"".to_string()));

            bars.push(bar);
        }

        run_native(
            "bar_graph",
            NativeOptions::default(),
            Box::new(|_cc| Box::new(BarGraphGui::new(bars))),
        )
        .unwrap();

        Ok(Value::Empty)
    }
}

struct BarGraphGui {
    bars: Vec<Bar>,
}

impl BarGraphGui {
    fn new(data: Vec<Bar>) -> Self {
        Self { bars: data }
    }
}

impl eframe::App for BarGraphGui {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            EguiPlot::new("bar_graph").show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new(self.bars.clone()).color(Color32::RED));
            });
        });
    }
}

pub struct Plot;

impl Tool for Plot {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "plot",
            description: "Render a list of numbers as a scatter plot graph.",
            group: "gui",
            inputs: vec![
                ValueType::ListOf(Box::new(ValueType::Float)),
                ValueType::ListOf(Box::new(ValueType::Integer)),
            ],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_list()?;
        let mut floats = Vec::new();

        for value in argument {
            if let Ok(float) = value.as_float() {
                floats.push(float);
            } else if let Ok(integer) = value.as_int() {
                floats.push(integer as f64);
            } else {
                return Err(Error::expected_number(value.clone()));
            }
        }

        run_native(
            "plot",
            NativeOptions {
                resizable: true,
                centered: true,
                ..Default::default()
            },
            Box::new(|_cc| Box::new(PlotGui::new(floats))),
        )
        .unwrap();

        Ok(Value::Empty)
    }
}

struct PlotGui {
    data: Vec<f64>,
}

impl PlotGui {
    fn new(data: Vec<f64>) -> Self {
        Self { data }
    }
}

impl eframe::App for PlotGui {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            EguiPlot::new("plot").show(ui, |plot_ui| {
                let points = self
                    .data
                    .iter()
                    .enumerate()
                    .map(|(index, value)| [index as f64, *value])
                    .collect::<PlotPoints>();
                let line = Line::new(points);
                plot_ui.line(line);
            })
        });
    }
}

pub struct GuiApp {
    text_edit_buffer: String,
    _whale_context: VariableMap,
    eval_result: Result<Value>,
}

impl GuiApp {
    pub fn new(result: Result<Value>) -> Self {
        GuiApp {
            text_edit_buffer: String::new(),
            _whale_context: VariableMap::new(),
            eval_result: result,
        }
    }

    fn _table_ui(&mut self, table: &Table, ui: &mut Ui) {
        TableBuilder::new(ui)
            .resizable(true)
            .striped(true)
            .columns(Column::remainder(), table.column_names().len())
            .header(30.0, |mut row| {
                for name in table.column_names() {
                    row.col(|ui| {
                        ui.label(name);
                    });
                }
            })
            .body(|body| {
                body.rows(20.0, table.rows().len(), |index, mut row| {
                    let row_data = table.rows().get(index).unwrap();

                    for cell_data in row_data {
                        row.col(|ui| {
                            ui.label(cell_data.to_string());
                        });
                    }
                });
            });
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout {
                    main_dir: Direction::TopDown,
                    main_wrap: false,
                    main_align: Align::Center,
                    main_justify: false,
                    cross_align: Align::Center,
                    cross_justify: true,
                },
                |ui| {
                    ui.text_edit_multiline(&mut self.text_edit_buffer);
                    ui.horizontal(|ui| {
                        let clear = ui.button("clear");
                        let submit = ui.button("submit");

                        if clear.clicked() {
                            self.text_edit_buffer.clear();
                        }

                        if submit.clicked() {
                            todo!()
                        }
                    });
                },
            );
            ui.separator();

            StripBuilder::new(ui)
                .sizes(egui_extras::Size::remainder(), 1)
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        let rectangle = ui.available_rect_before_wrap();
                        let corner = 5.0;
                        let border = Stroke::new(1.0, Color32::DARK_GREEN);
                        let item_size = 20.0;

                        ui.painter().rect_stroke(rectangle, corner, border);

                        match &self.eval_result {
                            Ok(value) => match value {
                                Value::String(string) => {
                                    ui.label(RichText::new(string).size(item_size));
                                }
                                Value::Float(float) => {
                                    ui.label(RichText::new(float.to_string()).size(item_size));
                                }
                                Value::Integer(integer) => {
                                    ui.label(RichText::new(integer.to_string()).size(item_size));
                                }
                                Value::Boolean(boolean) => {
                                    ui.label(RichText::new(boolean.to_string()).size(item_size));
                                }
                                Value::List(list) => {
                                    for value in list {
                                        ui.label(RichText::new(value.to_string()).size(item_size));
                                    }
                                }
                                Value::Map(_) => todo!(),
                                Value::Table(table) => {
                                    ScrollArea::both().show(ui, |ui| {
                                        TableBuilder::new(ui)
                                            .resizable(true)
                                            .striped(true)
                                            .vscroll(true)
                                            .columns(
                                                Column::remainder(),
                                                table.column_names().len(),
                                            )
                                            .header(20.0, |mut row| {
                                                for name in table.column_names() {
                                                    row.col(|ui| {
                                                        ui.label(name);
                                                    });
                                                }
                                            })
                                            .body(|body| {
                                                body.rows(
                                                    20.0,
                                                    table.rows().len(),
                                                    |index, mut row| {
                                                        let row_data =
                                                            table.rows().get(index).unwrap();

                                                        for value in row_data {
                                                            row.col(|ui| {
                                                                ui.label(value.to_string());
                                                            });
                                                        }
                                                    },
                                                );
                                            });
                                    });
                                }
                                Value::Function(_) => todo!(),
                                Value::Empty => {}
                                Value::Time(_) => todo!(),
                            },
                            Err(error) => {
                                let rectangle = ui.available_rect_before_wrap();
                                let corner = 5.0;
                                let border = Stroke::new(1.0, Color32::DARK_RED);

                                ui.painter().rect_stroke(rectangle, corner, border);
                                ui.label(error.to_string());
                            }
                        }
                    });
                });
        });
    }
}
