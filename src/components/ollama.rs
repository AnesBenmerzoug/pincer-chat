use futures::{FutureExt, StreamExt, TryStreamExt};

use relm4::{Component, ComponentParts, ComponentSender};
use tracing;

use crate::ollama::{
    client::{chat, pull_model},
    types::{ChatResponse, Message, PullModelResponse, Role},
};

#[derive(Debug)]
pub struct OllamaComponent {
    messages: Vec<Message>,
}

#[derive(Debug)]
pub enum OllamaInputMsg {
    Pull(String),
    Chat(String, Message),
}

#[derive(Debug)]
pub enum OllamaOutputMsg {
    PulledModel(String),
    Answer(Message),
}

#[derive(Debug)]
pub enum OllamaCmdMsg {
    ChatAnswer(Message),
    PulledModel(String),
}

impl Component for OllamaComponent {
    type Init = ();
    type Input = OllamaInputMsg;
    type Output = OllamaOutputMsg;
    type Root = ();
    type Widgets = ();
    type CommandOutput = OllamaCmdMsg;

    fn init_root() -> Self::Root {}

    fn init(_: Self::Init, _: Self::Root, _: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = OllamaComponent {
            messages: Vec::new(),
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        tracing::debug!("Ollama component update");

        match msg {
            OllamaInputMsg::Chat(model, message) => {
                tracing::info!("received message: {:?}", message);
                self.messages.push(message);
                let messages = self.messages.clone();

                sender.command(
                    |out: relm4::Sender<OllamaCmdMsg>, shutdown: relm4::ShutdownReceiver| {
                        shutdown
                            .register(async move {
                                let mut response_stream =
                                    match chat(model, messages)
                                        .await {
                                            Ok(response_stream) => response_stream,
                                            Err(e) => {
                                                tracing::error!("Failed generating answer {}", e);
                                                out.send(OllamaCmdMsg::ChatAnswer(Message {
                                                    content: "I am sorry. I am having issues generating an answer".to_string(),
                                                    role: Role::Assistant,
                                                })).expect("Message to be sent in channel");
                                                return;
                                            },
                                        };
                                let mut answer = String::new();
                                loop {
                                    match response_stream.next().await {
                                        Some(chat_response) => match chat_response {
                                            Ok(chat_response) => {
                                                if chat_response.done != true {
                                                    let answer_delta = &*chat_response.message.content;
                                                    tracing::info!("Received answer delta {}", answer_delta);
                                                    answer.push_str(answer_delta);
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Error during chat response generation {}", e);
                                                break;
                                            },
                                        },
                                        None => {
                                            tracing::warn!("No chat response");
                                            break;
                                        },
                                    };
                                }
                                out.send(OllamaCmdMsg::ChatAnswer(Message {
                                    content: answer,
                                    role: Role::Assistant,
                                }))
                                .unwrap();
                            })
                            // Perform task until a shutdown interrupts it
                            .drop_on_shutdown()
                            // Wrap into a `Pin<Box<Future>>` for return
                            .boxed()
                    },
                )
            }
            OllamaInputMsg::Pull(model) => {
                tracing::info!("pulling model: {}", model);

                sender.command(
                    |out: relm4::Sender<OllamaCmdMsg>, shutdown: relm4::ShutdownReceiver| {
                        shutdown
                            .register(async move {
                                let mut response_stream = match pull_model(model.clone()).await {
                                    Ok(response_stream) => response_stream,
                                    Err(e) => {
                                        tracing::error!("Failed pulling model {}", e);
                                        return;
                                    }
                                };
                                loop {
                                    match response_stream.next().await {
                                        Some(pull_response) => match pull_response {
                                            Ok(pull_response) => {
                                                tracing::debug!(
                                                    "pull model status: {:?}",
                                                    pull_response
                                                );
                                            }
                                            Err(_) => break,
                                        },
                                        None => break,
                                    };
                                }
                                out.send(OllamaCmdMsg::PulledModel(model)).unwrap();
                            })
                            // Perform task until a shutdown interrupts it
                            .drop_on_shutdown()
                            // Wrap into a `Pin<Box<Future>>` for return
                            .boxed()
                    },
                )
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        tracing::debug!("Ollama component update_");
        match message {
            OllamaCmdMsg::PulledModel(model) => {
                sender
                    .output(OllamaOutputMsg::PulledModel(model))
                    .expect("Message to be sent to App");
            }
            OllamaCmdMsg::ChatAnswer(message) => {
                self.messages.push(message.clone());
                sender
                    .output(OllamaOutputMsg::Answer(message))
                    .expect("Message to be sent to App");
            }
        }
    }
}
