mod assistant;

use std::str::FromStr;

use iced::padding;
use iced::widget::{button, center, column, container, row, scrollable, text, text_editor};
use iced::{Center, Element, Fill, Task, Theme};

use assistant::{Assistant, AssistantMessages, AssistantMessage};

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .run_with(App::new)
}
#[derive(Debug)]
struct App {
    state: State,
    input: text_editor::Content,
    messages: AssistantMessages,
    assistant: Assistant,
}

#[derive(Debug)]
enum State {
    Loading,
    Running,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(text_editor::Action),
    SubmitMessage,
}

impl App {
    fn title(&self) -> String {
        return String::from("Rusty Local AI Assistant");
    }

    pub fn new() -> (Self, Task<Message>) {
        let model = String::from("llama3.2");
        let mut messages = AssistantMessages::new();
        messages
            .add_system_message(
                String::from_str("You are a helpful AI assistant called Rusty.").unwrap(),
            );
        (
            Self {
                state: State::Running,
                input: text_editor::Content::new(),
                messages: messages,
                assistant: Assistant::new(model),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(action) => {
                self.input.perform(action);
                Task::none()
            }
            Message::SubmitMessage => {
                if self.input.text().trim().is_empty() {
                    Task::none()
                } else {
                    if let State::Running = &mut self.state {
                        self.messages.add_user_message(self.input.text());
                        let output_message = self.assistant.generate_answer(&self.messages);
                        self.messages.add_message(output_message);
                        // Empty the input field
                        self.input = text_editor::Content::new();
                    }
                    Task::none()
                }
            }
        }
    }

    fn view(& self) -> Element<Message> {
        let messages: Element<_> = if self.messages.is_empty() {
            center(
                match &self.state {
                    State::Running { .. } => column![text("Your assistant is ready."),],
                    State::Loading { .. } => column![
                        text("Your assistant is launching..."),
                        text("You can begin typing while you wait! ↓").style(text::success),
                    ],
                }
                .spacing(10)
                .align_x(Center),
            )
            .into()
        } else {
            scrollable(column(self.messages.messages.iter().map(message_bubble)).spacing(10))
                .anchor_y(scrollable::Anchor::End)
                .height(Fill)
                .into()
        };

        let input_text = text_editor(&self.input)
            .placeholder("Type your message here...")
            .padding(10)
            .on_action(Message::InputChanged)
            .key_binding(|key_press| {
                let modifiers = key_press.modifiers;

                match text_editor::Binding::from_key_press(key_press) {
                    Some(text_editor::Binding::Enter) if !modifiers.shift() => {
                        Some(text_editor::Binding::Custom(Message::SubmitMessage))
                    }
                    binding => binding,
                }
            });

        let submit_button = button(text("Submit")).on_press(Message::SubmitMessage);

        let input = row![input_text, submit_button];

        let chat = column![messages, input].spacing(10).align_x(Center);
        chat.into()
    }
}

fn message_bubble(message: &AssistantMessage) -> Element<Message> {
    use iced::border;

    let bubble = container(
        container(text(message.content.clone()).shaping(text::Shaping::Advanced))
            .width(Fill)
            .style(move |theme: &Theme| {
                let palette = theme.extended_palette();

                let (background, radius) =
                    (palette.success.weak, border::radius(10.0).top_right(0));

                container::Style {
                    background: Some(background.color.into()),
                    text_color: Some(background.text),
                    border: border::rounded(radius),
                    ..container::Style::default()
                }
            })
            .padding(10),
    )
    .padding(padding::left(20));

    bubble.into()
}
