mod components;
mod ollama;

use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use tracing;

use crate::components::ollama::OllamaComponent;
use crate::components::ollama::{OllamaInputMsg, OllamaOutputMsg};
use crate::ollama::types::{Message, Role};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";
const NO_MODEL_DROP_DOWN_VALUE: &str = "-";

#[derive(Debug)]
struct App {
    messages: FactoryVecDeque<MessageComponent>,
    user_input: gtk::EntryBuffer,
    model: Option<String>,
    ollama: Controller<OllamaComponent>,
}

#[derive(Debug)]
enum AppInputMsg {
    SelectModel(String),
    PulledModel(String),
    ReceivedAnswer(Message),
    Submit,
}

#[derive(Debug)]
enum AppOutputMsg {
    Submit(Message),
}

#[derive(Debug)]
struct MessageComponent {
    message: Message,
}

#[relm4::factory]
impl FactoryComponent for MessageComponent {
    type Init = Message;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Text {
            set_text: &self.message.content,
            add_css_class: match self.message.role {
                Role::System => "system_message",
                Role::User => "user_message",
                Role::Assistant => "assistant_message",
                Role::Tool => "tool_message",
            }
        }
    }

    fn init_model(
        message: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { message }
    }
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppInputMsg;
    type Output = AppOutputMsg;

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Chat"),
            set_default_size: (800, 600),
            set_hexpand: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_all: 5,
                    set_spacing: 5,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,

                    gtk::Label {
                        set_label: "Model",
                    },
                    gtk::DropDown::from_strings(&[NO_MODEL_DROP_DOWN_VALUE, "deepseek-r1:1.5b", "deepseek-r1", "llama3.2:1b", "llama3.2"]) {
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                        connect_selected_notify[sender] => move |model_drop_down| {
                            sender.input(AppInputMsg::SelectModel(
                                model_drop_down.selected_item().unwrap().downcast::<gtk::StringObject>().unwrap().into()))
                        },
                    },
                },

                gtk::ScrolledWindow {
                    #[local]
                    factory_box -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 5,
                        set_spacing: 5,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_all: 5,
                    set_spacing: 5,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::End,

                    gtk::Entry {
                        set_buffer: &model.user_input,
                        set_tooltip_text: Some("Send a message"),
                        set_placeholder_text: Some("Send a message"),
                        connect_activate => AppInputMsg::Submit,
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                    },
                    gtk::Button {
                        set_label: "Send",
                        connect_clicked => AppInputMsg::Submit,
                    }
                }
            }
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let factory_box = gtk::Box::default();

        let messages = FactoryVecDeque::builder()
            .launch(factory_box.clone())
            .forward(sender.input_sender(), |_| AppInputMsg::Submit);

        let ollama =
            OllamaComponent::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    OllamaOutputMsg::Answer(answer) => AppInputMsg::ReceivedAnswer(answer),
                    OllamaOutputMsg::PulledModel(model) => AppInputMsg::PulledModel(model),
                });

        let model = App {
            messages: messages,
            user_input: gtk::EntryBuffer::default(),
            model: None,
            ollama: ollama,
        };

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppInputMsg::SelectModel(model) => {
                tracing::info!("selected model {}", model);
                if model != NO_MODEL_DROP_DOWN_VALUE {
                    self.ollama
                        .sender()
                        .send(OllamaInputMsg::Pull(model))
                        .expect("Message to be sent to Ollama Component");
                }
            }
            AppInputMsg::PulledModel(model) => {
                tracing::info!("pulled model {}", model);
                self.model = Some(model.clone());
            }
            AppInputMsg::Submit => {
                let text = self.user_input.text();
                if !text.is_empty() && self.model.is_some() {
                    tracing::info!("Submitting user input {}", text.to_string());
                    let message = Message {
                        content: text.to_string(),
                        role: Role::User,
                    };
                    self.ollama
                        .sender()
                        .send(OllamaInputMsg::Chat(
                            self.model.clone().expect("Model to be set"),
                            message.clone(),
                        ))
                        .expect("Message to be sent to Ollama Component");
                    let mut guard = self.messages.guard();
                    guard.push_back(message);
                    // clearing the entry value clears the entry widget
                    self.user_input.set_text("");
                }
            }
            AppInputMsg::ReceivedAnswer(answer) => {
                let mut guard = self.messages.guard();
                guard.push_back(answer);
            }
        }
    }
}

fn main() {
    // Show traces to find potential performance bottlenecks, for example
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::TRACE)
        .init();

    tracing::info!("Starting application!");

    let relm = RelmApp::new(APP_ID);
    relm4::set_global_css_from_file("assets/style.css").expect("Expected a stylesheet");
    relm.run::<App>(());
}
