use futures::{FutureExt, StreamExt};
use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing;

use crate::assistant::ollama::types::{ChatResponse, Message, PullModelResponse, Role};
use crate::assistant::{Assistant, AssistantParameters};
use crate::components::chat_input::{ChatInputComponent, ChatInputInputMsg, ChatInputOutputMsg};
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};

#[derive(Debug)]
pub struct ChatScreen {
    assistant: Arc<Mutex<Assistant>>,
    options: AssistantOptions,
    // Components
    message_bubbles: Controller<MessageBubbleContainerComponent>,
    chat_input: Controller<ChatInputComponent>,
}

#[derive(Debug)]
pub struct AssistantOptions {
    pub temperature: f64,
    pub top_k: u64,
    pub top_p: f64,
}

impl Default for AssistantOptions {
    fn default() -> Self {
        Self {
            temperature: 0.5,
            top_k: 40,
            top_p: 0.9,
        }
    }
}

#[derive(Debug)]
pub enum ChatScreenInputMsg {
    SelectModel(String),
    SubmitUserInput(String),
    AssistantAnswer,
    // Assistant Parameters
    Temperature(f64),
    TopK(u64),
    TopP(f64),
    ResetParameters,
}

#[derive(Debug)]
pub enum ChatScreenCmdMsg {
    PullModelEnd,
    AnswerEnd,
    AppendToMessage(Message),
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for ParameterSpinButton {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 5,
            set_spacing: 5,
            set_halign: gtk::Align::End,
            set_valign: gtk::Align::Start,
        }
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for ChatScreen {
    type Init = Arc<Mutex<Assistant>>;
    type Input = ChatScreenInputMsg;
    type Output = ();
    type CommandOutput = ChatScreenCmdMsg;

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
                        set_css_classes: &["dropdown", "model_dropdown"],

                        connect_selected_notify[sender] => move |model_drop_down| {
                            sender.input(ChatScreenInputMsg::SelectModel(
                                model_drop_down
                                .selected_item()
                                .expect("Getting selected item from dropdown should work")
                                .downcast::<gtk::StringObject>()
                                .expect("Conversion of gtk StringObject to String should work")
                                .into()))
                        },
                    },

                    gtk::MenuButton {
                        set_icon_name: "preferences-system-symbolic",
                        set_direction: gtk::ArrowType::Down,
                        set_css_classes: &["button", "options_menu_button"],

                        #[wrap(Some)]
                        set_popover: popover = &gtk::Popover {
                            set_position: gtk::PositionType::Bottom,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                // Temperature
                                #[template]
                                ParameterSpinButton {
                                    gtk::Label {
                                        set_label: "Temperature",
                                    },
                                    gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                                        #[watch]
                                        set_value: model.options.temperature,

                                        connect_value_changed[sender] => move |btn| {
                                            let value = btn.value();
                                            sender.input(ChatScreenInputMsg::Temperature(value));
                                        },
                                    },
                                },

                                // Top-K
                                #[template]
                                ParameterSpinButton {
                                    gtk::Label {
                                        set_label: "Top-K",
                                    },
                                    gtk::SpinButton::with_range(0.0, 100.0, 1.0) {
                                        #[watch]
                                        set_value: model.options.top_k as f64,

                                        connect_value_changed[sender] => move |btn| {
                                            let value = btn.value() as u64;
                                            sender.input(ChatScreenInputMsg::TopK(value));
                                        },
                                    },
                                },
                                
                                // Top-P
                                #[template]
                                ParameterSpinButton {
                                    gtk::Label {
                                        set_label: "Top-P",
                                    },
                                    gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                                        #[watch]
                                        set_value: model.options.top_p,

                                        connect_value_changed[sender] => move |btn| {
                                            let value = btn.value();
                                            sender.input(ChatScreenInputMsg::TopP(value));
                                        },
                                    },
                                },

                                gtk::Button {
                                    set_hexpand: true,
                                    set_halign: gtk::Align::Fill,
                                    set_icon_name: "edit-undo-symbolic",
                                    set_tooltip_text: Some("Restore default options"),
                                    set_css_classes: &["button", "reset_options_button"],
                                    connect_clicked => ChatScreenInputMsg::ResetParameters,
                                },
                            },
                        },
                    },
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
        let message_bubbles = MessageBubbleContainerComponent::builder()
            .launch(())
            .detach();

        let chat_input =
            ChatInputComponent::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    ChatInputOutputMsg::SubmitUserInput(message) => {
                        ChatScreenInputMsg::SubmitUserInput(message)
                    }
                });

        let model = ChatScreen {
            assistant: init,
            options: AssistantOptions::default(),
            message_bubbles,
            chat_input,
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

        {
            // System Message
            let message = Message {
                content: String::from("You are a helpful assistant. You reply to user queries in a helpful manner. \
                You should give concise responses to very simple questions, but provide thorough responses to more complex and open-ended questions. \
                You help with writing, analysis, question answering, math, coding, and all sorts of other tasks. \
                You use markdown formatting for your replies."),
                role: Role::System,
            };
            model
                .message_bubbles
                .sender()
                .emit(MessageBubbleContainerInputMsg::AddMessage(message.clone()));

            let mut assistant = model.assistant.lock().await;
            assistant.add_message(message);
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
            ChatScreenInputMsg::Temperature(value) => {
                self.assistant.lock().await.set_temperature(value.clone());
                self.options.temperature = value;
            }
            ChatScreenInputMsg::TopK(value) => {
                self.assistant.lock().await.set_top_k(value.clone());
                self.options.top_k = value;
            }
            ChatScreenInputMsg::TopP(value) => {
                self.assistant.lock().await.set_top_p(value.clone());
                self.options.top_p = value;
            }
            ChatScreenInputMsg::ResetParameters => {
                tracing::info!("Resetting assistant parameters");
                let mut assistant = self.assistant.lock().await;
                assistant.reset_parameters();
                self.options = AssistantOptions::default();
            }
            ChatScreenInputMsg::SelectModel(model) => {
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
                            out.emit(ChatScreenCmdMsg::PullModelEnd);
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                })
            }
            ChatScreenInputMsg::SubmitUserInput(user_input) => {
                tracing::info!("Submitting user input");
                let message = Message {
                    content: user_input,
                    role: Role::User,
                };
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AddMessage(message.clone()));
                {
                    let mut assistant = self.assistant.lock().await;
                    assistant.add_message(message);
                }
                sender
                    .input_sender()
                    .emit(ChatScreenInputMsg::AssistantAnswer);
            }
            ChatScreenInputMsg::AssistantAnswer => {
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
                            let mut message_stream = match assistant.generate_answer().await {
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
                                        out.emit(ChatScreenCmdMsg::AppendToMessage(message));
                                    }
                                    Err(error) => {
                                        tracing::error!(
                                            "Error receiving assistant answer because of: {error}"
                                        );
                                        return;
                                    }
                                }
                            }
                            out.emit(ChatScreenCmdMsg::AnswerEnd);
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
            ChatScreenCmdMsg::PullModelEnd => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
            ChatScreenCmdMsg::AppendToMessage(message) => {
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AppendToLastMessage(message));
            }
            ChatScreenCmdMsg::AnswerEnd => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
        }
    }
}
