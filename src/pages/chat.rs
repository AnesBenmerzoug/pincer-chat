use futures::FutureExt;
use gtk::prelude::*;
use relm4::prelude::*;
use std::sync::mpsc;
use tokio::time::Duration;
use tracing;

use crate::assistant::ollama::types::{ChatResponse, Message, PullModelResponse, Role};
use crate::components::chat_input::{ChatInputComponent, ChatInputInputMsg, ChatInputOutputMsg};
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};

#[derive(Debug)]
pub struct ChatPage {
    // Components
    message_bubbles: Controller<MessageBubbleContainerComponent>,
    chat_input: Controller<ChatInputComponent>,
}

#[derive(Debug)]
pub enum ChatPageInputMsg {
    SelectModel(String),
    PullModelResponse(mpsc::Receiver<Option<PullModelResponse>>),
    SubmitUserInput(String),
    AssistantAnswer(mpsc::Receiver<Option<ChatResponse>>),
}

#[derive(Debug)]
pub enum ChatPageCmdMsg {
    PullModelDone,
    ChatDone,
    AppendToMessage(Message),
}

#[derive(Debug)]
pub enum ChatPageOutputMsg {
    TriggerModelPull(String),
    GetAssistantAnswer(Message),
}

#[relm4::component(pub)]
impl Component for ChatPage {
    type Init = ();
    type Input = ChatPageInputMsg;
    type Output = ChatPageOutputMsg;
    type CommandOutput = ChatPageCmdMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_css_classes: &["main_container"],

            // Message bubbles
            #[local_ref]
            message_bubbles -> gtk::Box{},

            // User Chat Input Fields
            #[local_ref]
            chat_input -> gtk::Box {},
        },
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let message_bubbles = MessageBubbleContainerComponent::builder()
            .launch(())
            .detach();

        let chat_input =
            ChatInputComponent::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    ChatInputOutputMsg::SubmitUserInput(message) => {
                        ChatPageInputMsg::SubmitUserInput(message)
                    }
                });

        let model = ChatPage {
            message_bubbles,
            chat_input,
        };

        // References used in the view macro
        let message_bubbles = model.message_bubbles.widget();
        let chat_input = model.chat_input.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        match message {
            ChatPageInputMsg::SelectModel(model) => {
                self.chat_input.sender().emit(ChatInputInputMsg::Disable);
                sender
                    .output_sender()
                    .emit(ChatPageOutputMsg::TriggerModelPull(model));
            }
            ChatPageInputMsg::PullModelResponse(receiver) => {
                sender.command(|out, shutdown: relm4::ShutdownReceiver| {
                    shutdown
                        .register(async move {
                            loop {
                                match receiver.try_recv() {
                                    Ok(response) => match response {
                                        Some(response) => tracing::info!(
                                            "Received pull model response: {:?}",
                                            response
                                        ),
                                        None => {
                                            tracing::info!(
                                                "Finished receiving pull model response"
                                            );
                                            out.emit(ChatPageCmdMsg::PullModelDone);
                                            break;
                                        }
                                    },
                                    Err(error) => {
                                        match error {
                                            mpsc::TryRecvError::Empty => {
                                                tokio::time::sleep(Duration::from_millis(100)).await;
                                            },
                                            mpsc::TryRecvError::Disconnected => {
                                                tracing::error!("Error receiving pull model response because of: {error}");
                                                break;
                                            },
                                        }
                                    }
                                }
                            }
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                })
            }
            ChatPageInputMsg::SubmitUserInput(user_input) => {
                let message = Message {
                    content: user_input,
                    role: Role::User,
                };
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AddMessage(message.clone()));
                sender
                    .output_sender()
                    .emit(ChatPageOutputMsg::GetAssistantAnswer(message));
            }
            ChatPageInputMsg::AssistantAnswer(receiver) => {
                let message = Message {
                    content: String::new(),
                    role: Role::Assistant,
                };
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AddMessage(message));
                sender.command(|out, shutdown: relm4::ShutdownReceiver| {
                    shutdown
                        .register(async move {
                            loop {
                                match receiver.try_recv() {
                                    Ok(response) => match response {
                                        Some(response) => {
                                            tracing::info!(
                                                "Received assistant answer: {:?}",
                                                response
                                            );
                                            out.emit(ChatPageCmdMsg::AppendToMessage(response.message));
                                        },
                                        None => {
                                            tracing::info!("Finished receiving assistant answer");
                                            out.emit(ChatPageCmdMsg::ChatDone);
                                            break;
                                        }
                                    },
                                    Err(error) => {
                                        match error {
                                            mpsc::TryRecvError::Empty => {
                                                tokio::time::sleep(Duration::from_millis(100)).await;
                                            },
                                            mpsc::TryRecvError::Disconnected => {
                                                tracing::error!("Error receiving assistant answer because of: {error}");
                                                break;
                                            },
                                        }
                                    }
                                }
                            }
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                })
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ChatPageCmdMsg::PullModelDone => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
            ChatPageCmdMsg::AppendToMessage(message) => {
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AppendToLastMessage(message));
            }
            ChatPageCmdMsg::ChatDone => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
        }
    }
}
