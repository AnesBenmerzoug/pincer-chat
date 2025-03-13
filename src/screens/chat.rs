use futures::{FutureExt, StreamExt};
use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing;

use crate::assistant::ollama::types::{Message, Role};
use crate::assistant::{database::Database, notification::NotifierMessage, Assistant};
use crate::components::chat_input::{ChatInputComponent, ChatInputInputMsg, ChatInputOutputMsg};
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};
use crate::components::thread_list::{
    ThreadListContainerComponent, ThreadListContainerInputMsg, ThreadListContainerOutputMsg,
};

#[derive(Debug)]
pub struct ChatScreen {
    assistant: Arc<Mutex<Assistant>>,
    chat_history: Arc<Mutex<Database>>,
    options: AssistantOptions,
    current_thread_id: i64,
    // Components
    message_bubbles: AsyncController<MessageBubbleContainerComponent>,
    chat_input: Controller<ChatInputComponent>,
    thread_list: AsyncController<ThreadListContainerComponent>,
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
    CreateNewThread,
    GetThreadMessages(i64),
    SubmitUserInput(String),
    DeleteThread(i64),
    AssistantAnswer,
    // Assistant Parameters
    SelectModel(String),
    Temperature(f64),
    TopK(u64),
    TopP(f64),
    ResetParameters,
}

#[derive(Debug)]
pub enum ChatScreenCmdMsg {
    PullModelEnd,
    AnswerEnd,
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
    type Init = (Arc<Mutex<Assistant>>, Arc<Mutex<Database>>);
    type Input = ChatScreenInputMsg;
    type Output = ();
    type CommandOutput = ChatScreenCmdMsg;

    view! {
        gtk::Paned {
            #[wrap(Some)]
            set_start_child = &gtk::Box{
                set_vexpand: true,
                set_hexpand: true,
                set_valign: gtk::Align::Fill,
                set_margin_all: 5,
                set_spacing: 5,
                set_css_classes: &["main_container"],

                #[local_ref]
                thread_list -> gtk::Box {},
            },

            #[wrap(Some)]
            set_end_child = &gtk::Box {
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
        },
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let assistant = init.0;
        let chat_history = init.1;

        let threads = {
            let mut chat_history = chat_history.lock().await;
            let mut threads = chat_history
                .get_threads()
                .await
                .expect("Getting all thread should work");

            if threads.len() == 0 {
                tracing::info!("No threads were found. Creating new one");
                let thread = chat_history
                    .create_thread("New Thread")
                    .await
                    .expect("Creating thread should work");
                threads.push(thread);
            }
            threads
        };

        let latest_thread = threads.first().expect("First thread must exist");
        let latest_thread_id = latest_thread.id;

        let messages = {
            let mut chat_history = chat_history.lock().await;
            chat_history
                .get_messages(latest_thread_id)
                .await
                .expect("Getting messages should work")
        };

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

        let thread_list = ThreadListContainerComponent::builder()
            .launch(threads)
            .forward(sender.input_sender(), |output| match output {
                ThreadListContainerOutputMsg::CreateNewThread => {
                    ChatScreenInputMsg::CreateNewThread
                }
                ThreadListContainerOutputMsg::GetThreadMessages(thread_id) => {
                    ChatScreenInputMsg::GetThreadMessages(thread_id)
                }
                ThreadListContainerOutputMsg::DeleteThread(thread_id) => {
                    ChatScreenInputMsg::DeleteThread(thread_id)
                }
            });

        let mut model = ChatScreen {
            assistant: assistant,
            chat_history: chat_history,
            options: AssistantOptions::default(),
            current_thread_id: latest_thread_id,
            message_bubbles,
            chat_input,
            thread_list,
        };

        // Connect chat history notifier to message bubbles
        {
            let chat_history = model.chat_history.lock().await;
            chat_history.notifier.subscribe(
                model.message_bubbles.sender(),
                |notifier_message: NotifierMessage| match notifier_message {
                    NotifierMessage::NewMessage(message) => {
                        Some(MessageBubbleContainerInputMsg::AddNewMessage(message))
                    }
                    NotifierMessage::UpdateMessage(message_update) => Some(
                        MessageBubbleContainerInputMsg::AppendToLastMessage(message_update),
                    ),
                    NotifierMessage::GetThreadMessages(messages) => {
                        Some(MessageBubbleContainerInputMsg::RefreshMessages(messages))
                    }
                    _ => None,
                },
            );
        }
        // Connect chat history notifier to thread list
        {
            let chat_history = model.chat_history.lock().await;
            chat_history.notifier.subscribe(
                model.thread_list.sender(),
                |notifier_message: NotifierMessage| match notifier_message {
                    NotifierMessage::NewThread(thread) => {
                        Some(ThreadListContainerInputMsg::AddThread(thread))
                    }
                    _ => None,
                },
            );
        }

        // References used in the view macro
        let message_bubbles = model.message_bubbles.widget();
        let chat_input = model.chat_input.widget();
        let thread_list = model.thread_list.widget();

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
            ChatScreenInputMsg::GetThreadMessages(thread_id) => {
                tracing::info!("Getting messages for thread with id {thread_id}");
                {
                    let mut chat_history = self.chat_history.lock().await;
                    chat_history
                        .get_messages(thread_id)
                        .await
                        .expect("Getting thread messages should work");
                    self.current_thread_id = thread_id;
                }
            }
            ChatScreenInputMsg::CreateNewThread => {
                tracing::info!("Creating new thread");
                let mut chat_history = self.chat_history.lock().await;
                let thread = chat_history
                    .create_thread("New Thread")
                    .await
                    .expect("Creating new thread should work");
                self.current_thread_id = thread.id;
            }
            ChatScreenInputMsg::DeleteThread(thread_id) => {
                tracing::info!("Deleting thread with id {thread_id}");
                let mut chat_history = self.chat_history.lock().await;
                chat_history
                    .delete_thread(thread_id)
                    .await
                    .expect("Deleting thread should work");
            }
            ChatScreenInputMsg::SubmitUserInput(user_input) => {
                tracing::info!("Submitting user input");
                let message = Message {
                    content: user_input,
                    role: Role::User,
                };
                {
                    let mut chat_history = self.chat_history.lock().await;
                    let thread_id = self.current_thread_id;
                    chat_history
                        .create_message(thread_id, message.content, message.role)
                        .await
                        .expect("Message should be created");
                }
                sender
                    .input_sender()
                    .emit(ChatScreenInputMsg::AssistantAnswer);
            }
            ChatScreenInputMsg::AssistantAnswer => {
                let thread_id = self.current_thread_id;
                let chat_history = self.chat_history.clone();

                let messages: Vec<Message> = {
                    let mut chat_history = chat_history.lock().await;
                    let messages = chat_history
                        .get_messages(thread_id)
                        .await
                        .expect("Getting messages should work");

                    messages
                        .into_iter()
                        .map(|m| Message {
                            content: m.content,
                            role: Role::try_from(m.role)
                                .expect("Role string to enum conversion should work"),
                        })
                        .collect()
                };

                let assistant_message_id = {
                    let mut chat_history = chat_history.lock().await;
                    let message = chat_history
                        .create_message(thread_id, String::new(), Role::Assistant)
                        .await
                        .expect("Creating an empty assistant message should work");
                    message.id
                };

                let assistant = self.assistant.clone();
                sender.command(move |out, shutdown: relm4::ShutdownReceiver| {
                    shutdown
                        .register(async move {
                            let mut assistant = assistant.lock().await;
                            let mut message_stream = match assistant.generate_answer(messages).await
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
                                        let mut chat_history = chat_history.lock().await;
                                        chat_history
                                            .update_message(assistant_message_id, message.content)
                                            .await
                                            .expect("Updating message in database should work");
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
            ChatScreenCmdMsg::AnswerEnd => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
        }
    }
}
