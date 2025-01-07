
use std::{thread, time};

use iced::{Center, Element, Fill, Font, Left, Right, Subscription, Task, Theme};
use iced::widget::{
    self, button, center, column, container, horizontal_space, hover, progress_bar, row,
    scrollable, stack, text, text_editor, tooltip, value,
};

pub fn main() -> iced::Result {
    iced::application(Assistant::TITLE, Assistant::update, Assistant::view)
        .run_with(Assistant::new)
}
#[derive(Debug)]
struct Assistant {
    state: State,
    input: text_editor::Content,
}

#[derive(Clone, Debug)]
enum State {
    Loading,
    Running,
}

#[derive(Debug, Clone)]
enum Message {
    LoadModel,
}


impl Assistant {
    const TITLE: &str = "Rusty";

    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
                input: text_editor::Content::new(),
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
        }
    }

    fn view(&self) -> Element<Message> {
        let messages: Element<_> = {
            center(
                match &self.state {
                    State::Running { .. } => column![
                        text("Your assistant is ready."),
                        text("Break the ice! ↓").style(text::primary),
                    ],
                    State::Loading { .. } => column![
                        text("Your assistant is launching..."),
                        text("You can begin typing while you wait! ↓").style(text::success),
                    ],
                }
                .spacing(10)
                .align_x(Center),
            )
            .into()
        };

        let input = text_editor(&self.input)
            .placeholder("Type your message here...")
            .padding(10);

        let chat = column![messages, input].spacing(10).align_x(Center);
        chat.into()
    }

}

