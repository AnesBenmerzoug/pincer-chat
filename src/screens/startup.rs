use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing;

use crate::assistant::ollama::types::{ChatResponse, Message, PullModelResponse, Role};
use crate::assistant::{Assistant, AssistantParameters};

#[derive(Debug)]
pub struct StartupPage {
    assistant: Arc<Mutex<Assistant>>,
    state: StartupPageState,
}

#[derive(Debug)]
pub enum StartupPageState {
    Start,
    CheckOllama,
    OllamaNotRunning,
    End,
}

#[derive(Debug)]
pub enum StartupPageInputMsg {
    Start,
    CheckOllamaIsRunning,
    OllamaNotRunning,
    Retry,
    ListModels,
    End,
}

#[derive(Debug)]
pub enum StartupPageOutputMsg {
    End,
}

#[relm4::component(async, pub)]
impl AsyncComponent for StartupPage {
    type Init = Arc<Mutex<Assistant>>;
    type Input = StartupPageInputMsg;
    type Output = StartupPageOutputMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::Fill,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                #[name = "spinner"]
                gtk::Spinner {
                    set_spinning: true,
                    set_margin_bottom: 10,
                },
                #[name = "status_label"]
                gtk::Label {
                    set_label: "Starting up application...",
                },
                #[name = "retry_button"]
                gtk::Button {
                    set_hexpand: false,
                    set_vexpand: false,
                    set_visible: false,
                    set_icon_name: "edit-undo-symbolic",
                    set_tooltip_text: Some("Retry"),
                    set_css_classes: &["reset_button"],
                    connect_clicked => StartupPageInputMsg::Retry,
                }
            },
        },
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = StartupPage {
            assistant: init,
            state: StartupPageState::Start,
        };

        let mut widgets = view_output!();

        model
            .update_with_view(&mut widgets, StartupPageInputMsg::Start, sender, &root)
            .await;

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        sleep(Duration::from_millis(100)).await;
        match message {
            StartupPageInputMsg::Start => {
                tracing::info!("Start up screen initialization");
                sender
                    .input_sender()
                    .emit(StartupPageInputMsg::CheckOllamaIsRunning);
            }
            StartupPageInputMsg::CheckOllamaIsRunning => {
                tracing::info!("Checking if Ollama is running");
                self.state = StartupPageState::CheckOllama;
                match self.assistant.lock().await.is_ollama_running().await {
                    true => {
                        tracing::info!("Ollama is running. Continuing");
                        sender.input_sender().emit(StartupPageInputMsg::ListModels);
                    }
                    false => {
                        tracing::error!("Ollama is not running. Stopping");
                        sender
                            .input_sender()
                            .emit(StartupPageInputMsg::OllamaNotRunning);
                    }
                }
            }
            StartupPageInputMsg::OllamaNotRunning => {
                tracing::error!("Ollama is not running. Waiting for user input");
                self.state = StartupPageState::OllamaNotRunning;
            }
            StartupPageInputMsg::Retry => {
                tracing::error!("User clicked retry button");
                widgets.retry_button.set_visible(false);
                widgets.spinner.set_spinning(true);
                sender
                    .input_sender()
                    .emit(StartupPageInputMsg::CheckOllamaIsRunning);
            }
            StartupPageInputMsg::ListModels => {
                tracing::info!("Listing local models");
                let mut assistant = self.assistant.lock().await;
                let models = match assistant.list_models().await {
                    Ok(models) => models,
                    Err(err) => {
                        tracing::error!(
                            "Could not retrieve list of local models because of: {err}"
                        );
                        Vec::new()
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
                assistant.set_model(model);
                sender.input_sender().emit(StartupPageInputMsg::End);
            }
            StartupPageInputMsg::End => {
                tracing::info!("Finished application startup");
                self.state = StartupPageState::End;
                sender.output_sender().emit(StartupPageOutputMsg::End);
            }
        }

        match self.state {
            StartupPageState::Start => {
                widgets.status_label.set_label("Starting up application...");
            }
            StartupPageState::CheckOllama => {
                widgets
                    .status_label
                    .set_label("Checking if Ollama is running...");
            }
            StartupPageState::OllamaNotRunning => {
                widgets
                    .status_label
                    .set_label("Ollama is not running. Please start it and try again");
                widgets.retry_button.set_visible(true);
                widgets.spinner.set_spinning(false);
            }
            StartupPageState::End => {
                widgets.status_label.set_label("Application is ready!");
            }
        }
    }
}
