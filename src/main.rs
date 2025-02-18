mod components;
mod ollama;
mod pages;

use std::time::Duration;

use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
    loading_widgets::LoadingWidgets,
    view, RelmApp,
};
use tokio;
use tracing;

use crate::components::ollama::OllamaComponent;
use crate::components::ollama::{OllamaInputMsg, OllamaOutputMsg};
use crate::ollama::types::Message;
use crate::pages::chat::{ChatPage, ChatPageInputMsg, ChatPageOutputMsg};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";

#[derive(Debug)]
struct App {
    state: AppState,
    model: Option<String>,
    // Components
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
enum AppMsg {
    ShowChatPage,
    PullModelStart(String),
    PullModelEnd(String),
    AssistantAnswerStart,
    AssistantAnswerChunk(Message),
    AssistantAnswerEnd,
    SendInputToAssistant(Message),
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Chat"),
            set_default_size: (800, 600),
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::Fill,

            #[local_ref]
            chat_page -> gtk::Box {},
        }
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local]
            root {
                set_title: Some("Chat"),
                set_default_size: (800, 600),
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,

                #[name = "widget"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    gtk::Spinner {
                        set_spinning: true,
                    },
                    gtk::Label {
                        set_label: "Starting up application...",
                    },
                },
            }
        }
        Some(LoadingWidgets::new(root, widget))
    }

    async fn init(
        _: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let chat_page = ChatPage::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                ChatPageOutputMsg::TriggerModelPull(model) => AppMsg::PullModelStart(model),
                ChatPageOutputMsg::GetAssistantAnswer(message) => {
                    AppMsg::SendInputToAssistant(message)
                }
            });

        let ollama =
            OllamaComponent::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    OllamaOutputMsg::PullModelEnd(model) => AppMsg::PullModelEnd(model),
                    OllamaOutputMsg::ChatAnswerStart => AppMsg::AssistantAnswerStart,
                    OllamaOutputMsg::ChatAnswerChunk(answer) => {
                        AppMsg::AssistantAnswerChunk(answer)
                    }
                    OllamaOutputMsg::ChatAnswerEnd => AppMsg::AssistantAnswerEnd,
                });

        let model = App {
            state: AppState::StartupPage,
            model: None,
            chat_page,
            ollama: ollama,
        };

        let chat_page = model.chat_page.widget();

        // Insert the macro code generation here
        let widgets = view_output! {};

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::ShowChatPage => {
                tracing::info!("Switching to Chat Page");
                self.state = AppState::ChatPage;
            }
            AppMsg::PullModelStart(model) => {
                tracing::info!("Pulling model {}", model);
                self.ollama
                    .sender()
                    .emit(OllamaInputMsg::PullModelStart(model));
                self.state = AppState::PullingModel;
            }
            AppMsg::PullModelEnd(model) => {
                tracing::info!("successfully pulled model");
                self.model = Some(model.clone());
                self.state = AppState::WaitingForUserInput;
                self.chat_page.sender().emit(ChatPageInputMsg::ModelReady);
            }
            AppMsg::SendInputToAssistant(message) => {
                tracing::info!("Sending user input to assistant");
                self.ollama.sender().emit(OllamaInputMsg::Chat(
                    self.model.clone().expect("Model to be set"),
                    message,
                ));
                self.state = AppState::ReceivingAnswer;
            }
            AppMsg::AssistantAnswerStart => {
                tracing::info!("Starting to receive answer");
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::AssistantAnswerStart);
            }
            AppMsg::AssistantAnswerChunk(answer) => {
                tracing::info!("Receiving answer");
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::AssistantAnswerProgress(answer));
            }
            AppMsg::AssistantAnswerEnd => {
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

    relm_app.run_async::<App>(());
}
