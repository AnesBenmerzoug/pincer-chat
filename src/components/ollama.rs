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
    PullModelStart(String),
    Chat(String, Message),
}

#[derive(Debug)]
pub enum OllamaCmdMsg {
    ChatAnswerStart,
    ChatAnswerChunk(Message),
    ChatAnswerEnd,
    PullModelEnd(String),
}

#[derive(Debug)]
pub enum OllamaOutputMsg {
    PullModelEnd(String),
    ChatAnswerStart,
    ChatAnswerChunk(Message),
    ChatAnswerEnd,
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

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        tracing::debug!("Ollama component update");

        match message {
            OllamaInputMsg::Chat(model, message) => {
                tracing::info!("received message: {:?}", message);
                self.messages.push(message);
                let messages = self.messages.clone();

                sender.command(
                    |out: relm4::Sender<OllamaCmdMsg>, shutdown: relm4::ShutdownReceiver| {
                        shutdown
                            .register(async move {
                                out.emit(OllamaCmdMsg::ChatAnswerStart);
                                let mut response_stream =
                                    match chat(model, messages)
                                        .await {
                                            Ok(response_stream) => response_stream,
                                            Err(e) => {
                                                tracing::error!("Failed generating answer {}", e);

                                                out.emit(OllamaCmdMsg::ChatAnswerChunk(Message {
                                                    content: "I am sorry. I am having issues generating an answer".to_string(),
                                                    role: Role::Assistant,
                                                }));
                                                return;
                                            },
                                        };
                                tracing::info!("Starting to receive chat answer");
                                loop {
                                    match response_stream.next().await {
                                        Some(chat_response) => match chat_response {
                                            Ok(chat_response) => {
                                                if chat_response.done != true {
                                                    let answer_delta = &*chat_response.message.content;
                                                    tracing::info!("Received answer delta {}", answer_delta);
                                                    out.emit(OllamaCmdMsg::ChatAnswerChunk(chat_response.message));
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
                                };
                                out.emit(OllamaCmdMsg::ChatAnswerEnd);
                            })
                            // Perform task until a shutdown interrupts it
                            .drop_on_shutdown()
                            // Wrap into a `Pin<Box<Future>>` for return
                            .boxed()
                    },
                )
            }
            OllamaInputMsg::PullModelStart(model) => {
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
                                out.emit(OllamaCmdMsg::PullModelEnd(model));
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
            OllamaCmdMsg::PullModelEnd(model) => {
                sender
                    .output_sender()
                    .emit(OllamaOutputMsg::PullModelEnd(model));
            }
            OllamaCmdMsg::ChatAnswerStart => {
                self.messages.push(Message {
                    content: String::new(),
                    role: Role::Assistant,
                });
                sender
                    .output_sender()
                    .emit(OllamaOutputMsg::ChatAnswerStart);
            }
            OllamaCmdMsg::ChatAnswerChunk(message_chunk) => {
                self.messages
                    .last_mut()
                    .expect("There should be a last element")
                    .update(&message_chunk)
                    .expect("The two messages must have the same role");
                sender
                    .output(OllamaOutputMsg::ChatAnswerChunk(message_chunk))
                    .expect("Message to be sent to App");
            }
            OllamaCmdMsg::ChatAnswerEnd => {
                sender
                    .output(OllamaOutputMsg::ChatAnswerEnd)
                    .expect("Message to be sent to App");
            }
        }
    }
}
