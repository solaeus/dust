use dust_lib::eval;
use iced::widget::{button, column, container, scrollable, text, text_input, Column, Text};
use iced::{executor, Alignment, Application, Command, Element, Sandbox, Settings, Theme};
use once_cell::sync::Lazy;

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn main() -> iced::Result {
    DustGui::run(Settings::default())
}

struct DustGui {
    text_buffer: String,
    results: Vec<String>,
}

impl Application for DustGui {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            DustGui {
                text_buffer: String::new(),
                results: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Dust".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::TextInput(input) => {
                self.text_buffer = input;

                Command::none()
            }
            Message::Evaluate => {
                let eval_result = eval(&self.text_buffer);

                match eval_result {
                    Ok(result) => self.results.push(result.to_string()),
                    Err(error) => self.results.push(error.to_string()),
                }

                Command::batch(vec![])
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let input = text_input("What needs to be done?", &self.text_buffer)
            .id(INPUT_ID.clone())
            .on_input(Message::TextInput)
            .on_submit(Message::Evaluate)
            .padding(15)
            .size(30);

        let result_display: Column<Message> = {
            let mut text_widgets = Vec::new();

            for result in &self.results {
                text_widgets.push(text(result).into());
            }

            text_widgets.reverse();

            Column::with_children(text_widgets)
        };

        container(column![input, result_display]).into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    TextInput(String),
    Evaluate,
}
