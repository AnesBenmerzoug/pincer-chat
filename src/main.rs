mod components;
mod ollama;

use gtk::prelude::*;
use relm4::prelude::*;
use tracing;

use crate::components::chat_input;
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};
use crate::components::ollama::OllamaComponent;
use crate::components::ollama::{OllamaInputMsg, OllamaOutputMsg};
use crate::ollama::types::{Message, Role};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";

#[derive(Debug)]
struct App {
    state: AppState,
    model: Option<String>,
    // Components
    message_bubbles: Controller<MessageBubbleContainerComponent>,
    chat_input: Controller<chat_input::ChatInputComponent>,
    ollama: Controller<OllamaComponent>,
}

#[derive(Debug, Clone)]
enum AppState {
    PullingModel,
    WaitingForUserInput,
    ReceivingAnswer,
}

#[derive(Debug)]
enum AppInputMsg {
    SelectModel(String),
    PulledModel(String),
    AssistantAnswerStart,
    AssistantAnswerChunk(Message),
    AssistantAnswerEnd,
    Submit(String),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppInputMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Chat"),
            set_default_size: (800, 600),
            set_hexpand: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
                set_css_classes: &["main_container"],

                // Model Selection
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_all: 5,
                    set_spacing: 5,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Start,

                    gtk::Label {
                        set_label: "Model",
                    },
                    #[name = "model_selection_drop_down"]
                    gtk::DropDown::from_strings(&["deepseek-r1:1.5b", "deepseek-r1", "llama3.2:1b", "llama3.2"]) {
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                        #[watch]
                        set_can_target: match model.state {
                            AppState::PullingModel | AppState::ReceivingAnswer => false,
                            _ => true,
                        },
                        connect_selected_notify[sender] => move |model_drop_down| {
                            sender.input(AppInputMsg::SelectModel(
                                model_drop_down
                                .selected_item()
                                .expect("Getting selected item from dropdown should work")
                                .downcast::<gtk::StringObject>()
                                .expect("Conversion of gtk StringObject to String should work")
                                .into()))
                        },
                    },
                },

                // Message bubbles
                #[local_ref]
                message_bubbles -> gtk::ScrolledWindow {},

                // User Chat Input Fields
                #[local_ref]
                chat_input -> gtk::Box {},
            }
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let message_bubbles = MessageBubbleContainerComponent::builder()
            .launch(())
            .detach();

        let ollama =
            OllamaComponent::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    OllamaOutputMsg::PulledModel(model) => AppInputMsg::PulledModel(model),
                    OllamaOutputMsg::ChatAnswerStart => AppInputMsg::AssistantAnswerStart,
                    OllamaOutputMsg::ChatAnswerChunk(answer) => {
                        AppInputMsg::AssistantAnswerChunk(answer)
                    }
                    OllamaOutputMsg::ChatAnswerEnd => AppInputMsg::AssistantAnswerEnd,
                });

        let chat_input = chat_input::ChatInputComponent::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                chat_input::OutputMsg::UserMessage(message) => AppInputMsg::Submit(message),
            });

        let model = App {
            state: AppState::WaitingForUserInput,
            message_bubbles,
            chat_input: chat_input,
            model: None,
            ollama: ollama,
        };

        // References used in the view macro
        let message_bubbles = model.message_bubbles.widget();
        let chat_input = model.chat_input.widget();

        // Insert the macro code generation here
        let widgets = view_output!();

        let default_model = widgets
            .model_selection_drop_down
            .selected_item()
            .unwrap()
            .downcast::<gtk::StringObject>()
            .unwrap()
            .into();
        sender.input(AppInputMsg::SelectModel(default_model));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _: ComponentSender<Self>) {
        match msg {
            AppInputMsg::SelectModel(model) => {
                tracing::info!("selected model {}", model);
                self.ollama
                    .sender()
                    .send(OllamaInputMsg::Pull(model))
                    .expect("Message to be sent to Ollama Component");
                self.state = AppState::PullingModel;
            }
            AppInputMsg::PulledModel(model) => {
                tracing::info!("pulled model {}", model);
                self.model = Some(model.clone());
                self.state = AppState::WaitingForUserInput;
            }
            AppInputMsg::Submit(message) => {
                let message = Message {
                    content: message,
                    role: Role::User,
                };

                self.message_bubbles
                    .sender()
                    .send(MessageBubbleContainerInputMsg::AddMessage(message.clone()))
                    .expect("Message to be sent to MessageBubble Container Component");

                tracing::info!("Sending user input to assistant");
                self.ollama
                    .sender()
                    .send(OllamaInputMsg::Chat(
                        self.model.clone().expect("Model to be set"),
                        message,
                    ))
                    .expect("Message to be sent to Ollama Component");
                self.state = AppState::ReceivingAnswer;
            }
            AppInputMsg::AssistantAnswerStart => {
                tracing::info!("Starting to receive answer");
                let message = Message {
                    content: String::new(),
                    role: Role::Assistant,
                };
                self.message_bubbles
                    .sender()
                    .send(MessageBubbleContainerInputMsg::AddMessage(message))
                    .expect("Message to be sent to MessageBubble Container Component");
            }
            AppInputMsg::AssistantAnswerChunk(answer) => {
                tracing::info!("Receiving answer");
                self.message_bubbles
                    .sender()
                    .send(MessageBubbleContainerInputMsg::ReplaceLastMessage(answer))
                    .expect("Message to be sent to MessageBubble Container Component");
            }
            AppInputMsg::AssistantAnswerEnd => {
                tracing::info!("Finished receiving answer");
                self.state = AppState::WaitingForUserInput;
            }
        }
    }
}

fn load_css(settings: &gtk::Settings) {
    let theme_name = settings
        .gtk_theme_name()
        .expect("Could not get theme name.");

    // Load common style sheet
    relm4::set_global_css_from_file("assets/common.css").expect("Expected a stylesheet");

    // Load mode-specific style sheet
    if theme_name.to_lowercase().contains("dark") || settings.is_gtk_application_prefer_dark_theme()
    {
        relm4::set_global_css_from_file("assets/dark.css").expect("Expected a stylesheet");
    } else {
        relm4::set_global_css_from_file("assets/light.css").expect("Expected a stylesheet");
    }
}

fn main() {
    // Show traces to find potential performance bottlenecks, for example
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::TRACE)
        .init();

    tracing::info!("Starting application!");
    let relm_app = RelmApp::new(APP_ID);

    let settings = gtk::Settings::default().expect("Accessing settings should work");
    settings.connect_gtk_application_prefer_dark_theme_notify(load_css);
    settings.connect_gtk_theme_name_notify(load_css);
    load_css(&settings);

    relm_app.run::<App>(());
}
