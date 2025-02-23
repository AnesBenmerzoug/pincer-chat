mod assistant;
mod components;
mod pages;

use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
    loading_widgets::LoadingWidgets,
    view, RelmApp,
};
use std::sync::mpsc;
use std::time::Duration;
use tokio;
use tracing;

use crate::assistant::{ollama::types::Message, Assistant, AssistantParameters};
use crate::pages::chat::{ChatPage, ChatPageInputMsg, ChatPageOutputMsg};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";

#[derive(Debug)]
struct App {
    assistant: Assistant,
    // Components
    chat_page: Controller<ChatPage>,
}

#[derive(Debug)]
enum AppMsg {
    PullModelRequest(String),
    GenerateAnswer(Message),
    SetAssistantOptions(AssistantParameters),
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
                        #[watch]
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
        dbg!(root.child());

        let mut assistant = Assistant::new();
        tracing::info!("Checking if Ollama is running");
        loop {
            match assistant.is_ollama_running().await {
                true => {
                    tracing::info!("Ollama is running");
                    break;
                }
                false => {
                    tracing::warn!("Ollama is not running. Waiting");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        let models = match assistant.list_models().await {
            Ok(models) => models,
            Err(err) => {
                tracing::error!("Could not retrieve list of local models because of: {err}");
                panic!("Could not retrieve list of local models");
            }
        };

        let mut model = String::from("llama3.2:1b");
        if models.len() == 0 {
            tracing::info!("No local model found. Using {model} as default model");
        } else {
            tracing::info!(
                "Found {} local models. Using {} as default model",
                models.len(),
                models[0]
            );
            model = models[0].clone();
        }

        assistant.set_model(model.clone());
        {
            tracing::info!("Pulling {model}");
            let (response_sender, _) = mpsc::channel();
            match assistant.pull_model(model, response_sender).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }

        let chat_page = ChatPage::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                ChatPageOutputMsg::TriggerModelPull(model) => AppMsg::PullModelRequest(model),
                ChatPageOutputMsg::GetAssistantAnswer(message) => AppMsg::GenerateAnswer(message),
                ChatPageOutputMsg::SetAssistantOptions(options) => {
                    AppMsg::SetAssistantOptions(options)
                }
            });

        let model = App {
            assistant,
            chat_page,
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
        tracing::info!("Update");
        match msg {
            AppMsg::PullModelRequest(model) => {
                tracing::info!("Pulling model {}", model);
                let (response_sender, response_receiver) = mpsc::channel();
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::PullModelResponse(response_receiver));
                match self
                    .assistant
                    .pull_model(model.clone(), response_sender)
                    .await
                {
                    Ok(_) => {
                        tracing::info!(
                            "Pulling model was successful. Setting {model} as current model"
                        );
                        self.assistant.set_model(model);
                    }
                    Err(error) => {
                        tracing::error!("Pulling model failed because of: {error}");
                    }
                }
            }
            AppMsg::GenerateAnswer(message) => {
                tracing::info!("Generating assistant answer");
                let (response_sender, response_receiver) = mpsc::channel();
                self.chat_page
                    .sender()
                    .emit(ChatPageInputMsg::AssistantAnswer(response_receiver));
                match self
                    .assistant
                    .generate_answer(message, response_sender)
                    .await
                {
                    Ok(_) => {
                        tracing::info!("Successfully generated assistant answer");
                    }
                    Err(error) => {
                        tracing::error!("Failed generating assistant answer because of: {error}");
                    }
                }
            }
            AppMsg::SetAssistantOptions(options) => {
                self.assistant.set_parameters(options);
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
