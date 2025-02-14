mod components;
mod ollama;
mod pages;

use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{RelmContainerExt, RelmRemoveExt};
use tracing;

use crate::components::ollama::OllamaComponent;
use crate::components::ollama::{OllamaInputMsg, OllamaOutputMsg};
use crate::ollama::types::Message;
use crate::pages::chat::{ChatPage, ChatPageInputMsg, ChatPageOutputMsg};
use crate::pages::startup::{StartUpPage, StartUpPageOutputMsg};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";

#[derive(Debug)]
struct App {
    state: AppState,
    model: Option<String>,
    // Components
    startup_page: Controller<StartUpPage>,
    chat_page: Controller<ChatPage>,
    ollama: Controller<OllamaComponent>,
}

#[derive(Debug, Clone)]
enum AppState {
    StartupPage,
    ChatPage,
    PullingModel,
    WaitingForUserInput,
    ReceivingAnswer,
}

#[derive(Debug)]
enum AppInputMsg {
    ShowChatPage,
    PullModelStart(String),
    PullModelEnd(String),
    AssistantAnswerStart,
    AssistantAnswerChunk(Message),
    AssistantAnswerEnd,
    SendInputToAssistant(Message),
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

            #[name = "page_container"]
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,

                // You can also use returned widgets
                #[name = "page_stack"]
                gtk::Stack {},
            },
        }
    }

    fn pre_view() {
        if let AppState::ChatPage = self.state {
            widgets.page_stack.set_visible_child_name("chat");
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let startup_page = StartUpPage::builder().launch(()).forward(
            sender.input_sender(),
            |output| match output {
                StartUpPageOutputMsg::Ready => AppInputMsg::ShowChatPage,
            },
        );

        let chat_page = ChatPage::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                ChatPageOutputMsg::TriggerModelPull(model) => AppInputMsg::PullModelStart(model),
                ChatPageOutputMsg::GetAssistantAnswer(message) => {
                    AppInputMsg::SendInputToAssistant(message)
                }
            });

        let ollama =
            OllamaComponent::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    OllamaOutputMsg::PullModelEnd(model) => AppInputMsg::PullModelEnd(model),
                    OllamaOutputMsg::ChatAnswerStart => AppInputMsg::AssistantAnswerStart,
                    OllamaOutputMsg::ChatAnswerChunk(answer) => {
                        AppInputMsg::AssistantAnswerChunk(answer)
                    }
                    OllamaOutputMsg::ChatAnswerEnd => AppInputMsg::AssistantAnswerEnd,
                });

        let model = App {
            state: AppState::StartupPage,
            model: None,
            startup_page,
            chat_page,
            ollama: ollama,
        };

        // Insert the macro code generation here
        let widgets = view_output! {};

        let startup_page = model.startup_page.widget();
        let chat_page = model.chat_page.widget();
        widgets.page_stack.add_named(startup_page, Some("startup"));
        widgets.page_stack.add_named(chat_page, Some("chat"));
        widgets.page_stack.set_visible_child_name("startup");

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _: ComponentSender<Self>) {
        match msg {
            AppInputMsg::ShowChatPage => {
                tracing::info!("Switching to Chat Page");
                self.state = AppState::ChatPage;
            }
            AppInputMsg::PullModelStart(model) => {
                tracing::info!("Pulling model {}", model);
                self.ollama
                    .sender()
                    .emit(OllamaInputMsg::PullModelStart(model));
                self.state = AppState::PullingModel;
            }
            AppInputMsg::PullModelEnd(model) => {
                tracing::info!("successfully pulled model");
                self.model = Some(model.clone());
                self.state = AppState::WaitingForUserInput;
                self.chat_page.sender().emit(ChatPageInputMsg::ModelReady);
            }
            AppInputMsg::SendInputToAssistant(message) => {
                tracing::info!("Sending user input to assistant");
                self.ollama.sender().emit(OllamaInputMsg::Chat(
                    self.model.clone().expect("Model to be set"),
                    message,
                ));
                self.state = AppState::ReceivingAnswer;
            }
            AppInputMsg::AssistantAnswerStart => {
                tracing::info!("Starting to receive answer");
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::AssistantAnswerStart);
            }
            AppInputMsg::AssistantAnswerChunk(answer) => {
                tracing::info!("Receiving answer");
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::AssistantAnswerProgress(answer));
            }
            AppInputMsg::AssistantAnswerEnd => {
                tracing::info!("Finished receiving answer");
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::AssistantAnswerEnd);
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
