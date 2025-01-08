use std::{thread, time};

use iced::clipboard;
use iced::padding;
use iced::widget::{
    self, button, center, column, container, horizontal_space, hover, progress_bar, row,
    scrollable, stack, text, text_editor, tooltip, value,
};
use iced::{Center, Element, Fill, Font, Left, Right, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(Assistant::TITLE, Assistant::update, Assistant::view).run_with(Assistant::new)
}
#[derive(Debug)]
struct Assistant {
    state: State,
    input: text_editor::Content,
    messages: Vec<String>,
}

#[derive(Clone, Debug)]
enum State {
    Loading,
    Running,
}

#[derive(Debug, Clone)]
enum Message {
    LoadModel,
    InputChanged(text_editor::Action),
    Submit,
    Done,
}

impl Assistant {
    const TITLE: &str = "Rusty";

    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
                input: text_editor::Content::new(),
                messages: Vec::new(),
            },
            Task::done(Message::LoadModel),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadModel => {
                let sleep_duration = time::Duration::from_secs(3);
                thread::sleep(sleep_duration);
                self.state = State::Running;
                Task::none()
            }
            Message::InputChanged(action) => {
                self.input.perform(action);
                Task::none()
            }
            Message::Submit => {
                if self.input.text().is_empty() {
                    Task::none()
                } else {
                    self.messages.push(self.input.text());
                    self.input = text_editor::Content::new();
                    Task::done(Message::Done)
                }
            }
            Message::Done => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
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
            scrollable(column(self.messages.iter().map(message_bubble)).spacing(10))
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
                        Some(text_editor::Binding::Custom(Message::Submit))
                    }
                    binding => binding,
                }
            });

        let submit_button = button(text("Submit")).on_press(Message::Submit);

        let input = row![input_text, submit_button];

        let chat = column![messages, input].spacing(10).align_x(Center);
        chat.into()
    }
}

fn message_bubble(message: &String) -> Element<Message> {
    use iced::border;

    let bubble = container(
        container(text(message).shaping(text::Shaping::Advanced))
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
