mod components;
mod ollama;

use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use tracing;

use crate::components::message::MessageComponent;
use crate::components::ollama::OllamaComponent;
use crate::components::ollama::{OllamaInputMsg, OllamaOutputMsg};
use crate::ollama::types::{Message, Role};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";

#[derive(Debug)]
struct App {
    state: AppState,
    messages: FactoryVecDeque<MessageComponent>,
    user_input: gtk::EntryBuffer,
    model: Option<String>,
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
    Submit,
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
                // Messages
                gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    #[local]
                    factory_box -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 10,
                        set_spacing: 10,
                        set_hexpand: false,
                        set_vexpand: true,
                        // set_halign: gtk::Align::Fill,
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
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                        #[watch]
                        set_can_target: match model.state {
                            AppState::PullingModel | AppState::ReceivingAnswer => false,
                            _ => true,
                        },
                        connect_activate => AppInputMsg::Submit,
                    },
                    gtk::Button {
                        set_label: "Send",
                        set_css_classes: &["submit_button"],
                        #[watch]
                        set_can_target: match model.state {
                            AppState::PullingModel | AppState::ReceivingAnswer => false,
                            _ => true,
                        },

                        connect_clicked[sender] => move |_| {
                            sender.input(AppInputMsg::Submit);
                        },
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
                    OllamaOutputMsg::PulledModel(model) => AppInputMsg::PulledModel(model),
                    OllamaOutputMsg::ChatAnswerStart => AppInputMsg::AssistantAnswerStart,
                    OllamaOutputMsg::ChatAnswerChunk(answer) => {
                        AppInputMsg::AssistantAnswerChunk(answer)
                    }
                    OllamaOutputMsg::ChatAnswerEnd => AppInputMsg::AssistantAnswerEnd,
                });

        let model = App {
            state: AppState::WaitingForUserInput,
            messages: messages,
            user_input: gtk::EntryBuffer::default(),
            model: None,
            ollama: ollama,
        };

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
            AppInputMsg::Submit => {
                let text = self.user_input.text();
                if !text.is_empty() && self.model.is_some() {
                    tracing::info!("Submitting user input {}", text.to_string());
                    let message = Message {
                        content: text.to_string(),
                        role: Role::User,
                    };
                    let mut guard = self.messages.guard();
                    guard.push_back(message.clone());
                    // clearing the entry value clears the entry widget
                    self.user_input.set_text("");

                    tracing::info!("Sending user input to assistant");
                    self.ollama
                        .sender()
                        .send(OllamaInputMsg::Chat(
                            self.model.clone().expect("Model to be set"),
                            message.clone(),
                        ))
                        .expect("Message to be sent to Ollama Component");
                    self.state = AppState::ReceivingAnswer;
                }
            }
            AppInputMsg::AssistantAnswerStart => {
                tracing::info!("Starting to receive answer");
                let mut guard = self.messages.guard();
                guard.push_back(Message {
                    content: String::new(),
                    role: Role::Assistant,
                });
            }
            AppInputMsg::AssistantAnswerChunk(answer) => {
                tracing::info!("Receiving answer chunk");
                let mut guard = self.messages.guard();
                guard
                    .back_mut()
                    .expect("There should be at least one previous message")
                    .replace_message(answer)
                    .expect("Replacing message should work");
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
