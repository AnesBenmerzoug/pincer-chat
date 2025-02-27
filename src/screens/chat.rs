use futures::{FutureExt, StreamExt};
use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing;

use crate::assistant::ollama::types::{ChatResponse, Message, PullModelResponse, Role};
use crate::assistant::{Assistant, AssistantParameters};
use crate::components::assistant_options_dialog::{
    AssistantOptionsDialog, AssistantOptionsDialogInputMsg, AssistantOptionsDialogOutputMsg,
};
use crate::components::chat_input::{ChatInputComponent, ChatInputInputMsg, ChatInputOutputMsg};
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};

#[derive(Debug)]
pub struct ChatPage {
    assistant: Arc<Mutex<Assistant>>,
    // Components
    message_bubbles: Controller<MessageBubbleContainerComponent>,
    chat_input: Controller<ChatInputComponent>,
    options_dialog: Controller<AssistantOptionsDialog>,
}

#[derive(Debug)]
pub enum ChatPageInputMsg {
    ShowOptionsDialog,
    SetAssistantOptions(AssistantParameters),
    SelectModel(String),
    SubmitUserInput(String),
    AssistantAnswer,
}

#[derive(Debug)]
pub enum ChatPageCmdMsg {
    PullModelEnd,
    AnswerEnd,
    AppendToMessage(Message),
}

#[derive(Debug)]
pub enum ChatPageOutputMsg {}

#[relm4::component(async, pub)]
impl AsyncComponent for ChatPage {
    type Init = Arc<Mutex<Assistant>>;
    type Input = ChatPageInputMsg;
    type Output = ChatPageOutputMsg;
    type CommandOutput = ChatPageCmdMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_css_classes: &["main_container"],

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

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
                    gtk::DropDown {
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                        connect_selected_notify[sender] => move |model_drop_down| {
                            sender.input(ChatPageInputMsg::SelectModel(
                                model_drop_down
                                .selected_item()
                                .expect("Getting selected item from dropdown should work")
                                .downcast::<gtk::StringObject>()
                                .expect("Conversion of gtk StringObject to String should work")
                                .into()))
                        },
                    }
                },

                #[name = "option_menu_button"]
                gtk::Button {
                    set_icon_name: "open-menu-symbolic",
                    set_icon_name: "preferences-system-symbolic",
                    set_css_classes: &["option_menu_button"],
                    connect_clicked => ChatPageInputMsg::ShowOptionsDialog,
                },
            },

            // Message bubbles
            #[local_ref]
            message_bubbles -> gtk::Box{},

            // User Chat Input Fields
            #[local_ref]
            chat_input -> gtk::Box {},
        },
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let options_dialog = AssistantOptionsDialog::builder()
            .transient_for(&root)
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                AssistantOptionsDialogOutputMsg::SendOptions(options) => {
                    ChatPageInputMsg::SetAssistantOptions(options)
                }
            });

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
            assistant: init,
            message_bubbles,
            chat_input,
            options_dialog,
        };

        // References used in the view macro
        let message_bubbles = model.message_bubbles.widget();
        let chat_input = model.chat_input.widget();

        let widgets = view_output!();

        {
            let assistant = model.assistant.lock().await;
            let local_models = match assistant.list_models().await {
                Ok(models) => models,
                Err(err) => {
                    tracing::error!("Could not retrieve list of local models because of: {err}");
                    Vec::new()
                }
            };

            let model_list = gtk::StringList::default();
            for model_name in local_models {
                model_list.append(&*model_name);
            }
            widgets
                .model_selection_drop_down
                .set_model(Some(&model_list));
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            ChatPageInputMsg::ShowOptionsDialog => {
                self.options_dialog
                    .sender()
                    .emit(AssistantOptionsDialogInputMsg::Show);
            }
            ChatPageInputMsg::SetAssistantOptions(options) => {
                self.chat_input.sender().emit(ChatInputInputMsg::Disable);
                self.assistant.lock().await.set_parameters(options);
            }
            ChatPageInputMsg::SelectModel(model) => {
                tracing::info!("Pulling model {model}");
                let assistant = self.assistant.clone();
                sender.command(|out, shutdown: relm4::ShutdownReceiver| {
                    shutdown
                        .register(async move {
                            let mut assistant = assistant.lock().await;
                            let mut response_stream = match assistant.pull_model(model.clone()).await {
                                Ok(stream) => stream,
                                Err(error) => {
                                    tracing::error!(
                                        "Error receiving pull model response because of: {error}"
                                    );
                                    return;
                                }
                            };

                            while let Some(result) = response_stream.next().await {
                                match result {
                                    Ok(pull_model_response) => {
                                        tracing::info!("Received pull model response: {:?}", pull_model_response);
                                    }
                                    Err(error) => {
                                        tracing::error!(
                                            "Error receiving pull model response because of: {error}"
                                        );
                                        return;
                                    }
                                }
                            }
                            assistant.set_model(model);
                            out.emit(ChatPageCmdMsg::PullModelEnd);
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                })
            }
            ChatPageInputMsg::SubmitUserInput(user_input) => {
                tracing::info!("Submitting user input");
                let message = Message {
                    content: user_input,
                    role: Role::User,
                };
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AddMessage(message.clone()));
                sender
                    .input_sender()
                    .emit(ChatPageInputMsg::AssistantAnswer);
            }
            ChatPageInputMsg::AssistantAnswer => {
                let message = Message {
                    content: String::new(),
                    role: Role::Assistant,
                };
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AddMessage(message.clone()));

                let assistant = self.assistant.clone();
                sender.command(|out, shutdown: relm4::ShutdownReceiver| {
                    shutdown
                        .register(async move {
                            let mut assistant = assistant.lock().await;
                            let mut message_stream = match assistant.generate_answer(message).await
                            {
                                Ok(stream) => stream,
                                Err(error) => {
                                    tracing::error!(
                                        "Error receiving assistant answer because of: {error}"
                                    );
                                    return;
                                }
                            };

                            while let Some(result) = message_stream.next().await {
                                match result {
                                    Ok(message) => {
                                        tracing::info!("Received assistant answer: {:?}", message);
                                        out.emit(ChatPageCmdMsg::AppendToMessage(message));
                                    }
                                    Err(error) => {
                                        tracing::error!(
                                            "Error receiving assistant answer because of: {error}"
                                        );
                                        return;
                                    }
                                }
                            }
                            out.emit(ChatPageCmdMsg::AnswerEnd);
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                })
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            ChatPageCmdMsg::PullModelEnd => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
            ChatPageCmdMsg::AppendToMessage(message) => {
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AppendToLastMessage(message));
            }
            ChatPageCmdMsg::AnswerEnd => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
        }
    }
}
