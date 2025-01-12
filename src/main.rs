mod assistant;

use iced::futures::channel::mpsc;
use iced::padding;
use iced::widget::{button, center, column, container, row, scrollable, text, text_editor};
use iced::{Center, Element, Fill, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    let model_repo_id = String::from("QuantFactory/Llama-3.2-1B-Instruct-GGUF");
    let model_file = String::from("Llama-3.2-1B-Instruct.Q6_K.gguf");
    let tokenizer_repo_id = String::from("meta-llama/Llama-3.2-1B-Instruct");

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || App::new(model_repo_id, model_file, tokenizer_repo_id))
}
#[derive(Debug)]
struct App {
    state: State,
    input: text_editor::Content,
    messages: Vec<String>,
    model_repo_id: String,
    model_file: String,
    tokenizer_repo_id: String,
}

#[derive(Debug)]
enum State {
    Loading,
    Running(mpsc::Sender<String>),
}

#[derive(Debug, Clone)]
enum Message {
    ReceivedMessage(assistant::Event),
    InputChanged(text_editor::Action),
    SubmitMessage,
}

impl App {
    fn title(&self) -> String {
        return String::from("Rusty Local AI Assistant");
    }

    pub fn new(
        model_repo_id: String,
        model_file: String,
        tokenizer_repo_id: String,
    ) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
                input: text_editor::Content::new(),
                messages: Vec::new(),
                model_repo_id,
                model_file,
                tokenizer_repo_id,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ReceivedMessage(event) => {
                match event {
                    assistant::Event::Loaded(sender) => {
                        self.state = State::Running(sender);
                    }
                    assistant::Event::GeneratedAnswerDelta(answer_delta) => {
                        println!("Received generated answer delta: {}", answer_delta);
                        let n_messages = self.messages.len();
                        self.messages[n_messages - 1].push_str(&answer_delta);
                    }
                    assistant::Event::FinishedGeneration => {
                        println!("Finished generation");
                    }
                }
                Task::none()
            }
            Message::InputChanged(action) => {
                self.input.perform(action);
                Task::none()
            }
            Message::SubmitMessage => {
                if self.input.text().trim().is_empty() {
                    Task::none()
                } else {
                    if let State::Running(sender) = &mut self.state {
                        if let Ok(()) = sender.try_send(self.input.text()) {
                            self.messages.push(self.input.text());
                            self.messages.push(String::from(""));
                            // Empty the input field
                            self.input = text_editor::Content::new();
                        }
                    }
                    Task::none()
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let model_repo_id = self.model_repo_id.clone();
        let model_file = self.model_file.clone();
        let tokenizer_repo_id = self.tokenizer_repo_id.clone();

        let start_assistant =
            move || assistant::start_assistant(model_repo_id, model_file, tokenizer_repo_id);
        Subscription::run_with_id("assistant_subscription", start_assistant())
            .map(Message::ReceivedMessage)
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
