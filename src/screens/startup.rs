use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing;

use crate::assistant::ollama::types::{ChatResponse, Message, PullModelResponse, Role};
use crate::assistant::{database::Database, Assistant, AssistantParameters};

#[derive(Debug)]
pub struct StartupScreen {
    assistant: Arc<Mutex<Assistant>>,
    chat_history: Arc<Mutex<Database>>,
    state: StartupScreenState,
}

#[derive(Debug)]
pub enum StartupScreenState {
    Start,
    CheckOllama,
    OllamaNotRunning,
    ListModels,
    RunningDatabaseMigrations,
    DatabaseMigrationsFailed,
    End,
}

#[derive(Debug)]
pub enum StartupScreenInputMsg {
    Start,
    CheckOllamaIsRunning,
    OllamaNotRunning,
    Retry,
    ListModels,
    RunDatabaseMigrations,
    DatabaseMigrationsFailed,
    End,
}

#[derive(Debug)]
pub enum StartupScreenOutputMsg {
    End,
}

#[relm4::component(async, pub)]
impl AsyncComponent for StartupScreen {
    type Init = (Arc<Mutex<Assistant>>, Arc<Mutex<Database>>);
    type Input = StartupScreenInputMsg;
    type Output = StartupScreenOutputMsg;
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
                    connect_clicked => StartupScreenInputMsg::Retry,
                }
            },
        },
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = StartupScreen {
            assistant: init.0,
            chat_history: init.1,
            state: StartupScreenState::Start,
        };

        let mut widgets = view_output!();

        model
            .update_with_view(&mut widgets, StartupScreenInputMsg::Start, sender, &root)
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
            StartupScreenInputMsg::Start => {
                tracing::info!("Start up screen initialization");
                sender
                    .input_sender()
                    .emit(StartupScreenInputMsg::CheckOllamaIsRunning);
                self.state = StartupScreenState::CheckOllama;
            }
            StartupScreenInputMsg::CheckOllamaIsRunning => {
                tracing::info!("Checking if Ollama is running");
                match self.assistant.lock().await.is_ollama_running().await {
                    true => {
                        tracing::info!("Ollama is running. Continuing");
                        sender
                            .input_sender()
                            .emit(StartupScreenInputMsg::ListModels);
                        self.state = StartupScreenState::ListModels;
                    }
                    false => {
                        tracing::error!("Ollama is not running. Stopping");
                        sender
                            .input_sender()
                            .emit(StartupScreenInputMsg::OllamaNotRunning);
                        self.state = StartupScreenState::OllamaNotRunning;
                    }
                }
            }
            StartupScreenInputMsg::OllamaNotRunning => {
                tracing::error!("Ollama is not running. Waiting for user input");
            }
            StartupScreenInputMsg::Retry => {
                tracing::error!("User clicked retry button");
                widgets.retry_button.set_visible(false);
                widgets.spinner.set_spinning(true);
                sender
                    .input_sender()
                    .emit(StartupScreenInputMsg::CheckOllamaIsRunning);
                self.state = StartupScreenState::CheckOllama;
            }
            StartupScreenInputMsg::ListModels => {
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
                sender
                    .input_sender()
                    .emit(StartupScreenInputMsg::RunDatabaseMigrations);
                self.state = StartupScreenState::RunningDatabaseMigrations;
            }
            StartupScreenInputMsg::RunDatabaseMigrations => {
                tracing::info!("Running database migrations");
                let chat_history = self.chat_history.lock().await;
                match chat_history.run_migrations().await {
                    Ok(_) => {
                        tracing::info!("Database migrations successful");
                        sender.input_sender().emit(StartupScreenInputMsg::End);
                        self.state = StartupScreenState::End;
                    }
                    Err(error) => {
                        tracing::error!("Database migrations failed because of {error}");
                        sender
                            .input_sender()
                            .emit(StartupScreenInputMsg::DatabaseMigrationsFailed);
                        self.state = StartupScreenState::DatabaseMigrationsFailed;
                    }
                }
            }
            StartupScreenInputMsg::DatabaseMigrationsFailed => {
                tracing::info!("Database migrations failed");
            }
            StartupScreenInputMsg::End => {
                tracing::info!("Finished application startup");
                self.state = StartupScreenState::End;
                sender.output_sender().emit(StartupScreenOutputMsg::End);
            }
        }

        match self.state {
            StartupScreenState::Start => {
                widgets.status_label.set_label("Starting up application...");
            }
            StartupScreenState::CheckOllama => {
                widgets
                    .status_label
                    .set_label("Checking if Ollama is running...");
            }
            StartupScreenState::OllamaNotRunning => {
                widgets
                    .status_label
                    .set_label("Ollama is not running :( Please start it and try again ");
                widgets.retry_button.set_visible(true);
                widgets.spinner.set_spinning(false);
            }
            StartupScreenState::ListModels => {
                widgets.status_label.set_label("Listing local models...");
            }
            StartupScreenState::RunningDatabaseMigrations => {
                widgets.status_label.set_label("Running migrations...");
            }
            StartupScreenState::DatabaseMigrationsFailed => widgets
                .status_label
                .set_label("Database migrations failed :("),
            StartupScreenState::End => {
                widgets.status_label.set_label("Application is ready!");
            }
        }
    }
}
