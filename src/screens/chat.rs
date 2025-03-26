use futures::{FutureExt, StreamExt};

use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing;

use crate::assistant::ollama::types::{Message, Role};
use crate::assistant::{database::Database, notification::DatabaseNotifierMessage, Assistant};
use crate::components::assistant_parameters::{
    AssistantParametersComponent, AssistantParametersOutputMsg,
};
use crate::components::chat_input::{ChatInputComponent, ChatInputOutputMsg};
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};
use crate::components::thread_list::{
    ThreadListContainerComponent, ThreadListContainerInputMsg, ThreadListContainerOutputMsg,
};

#[derive(Debug)]
pub struct ChatScreen {
    assistant: Arc<Mutex<Assistant>>,
    database: Arc<Mutex<Database>>,
    current_thread_id: i64,
    // Components
    assistant_parameters: Controller<AssistantParametersComponent>,
    thread_list: AsyncController<ThreadListContainerComponent>,
    chat_input: Controller<ChatInputComponent>,
    message_bubbles: AsyncController<MessageBubbleContainerComponent>,
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

impl ChatScreen {
    fn enable_inputs(&mut self) {
        self.assistant_parameters.widget().set_sensitive(true);
        self.thread_list.widget().set_sensitive(true);
        self.chat_input.widget().set_sensitive(true);
    }

    fn disable_inputs(&mut self) {
        self.assistant_parameters.widget().set_sensitive(false);
        self.thread_list.widget().set_sensitive(false);
        self.chat_input.widget().set_sensitive(false);
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
            set_vexpand: true,
            set_hexpand: true,
            set_valign: gtk::Align::Fill,
            set_halign: gtk::Align::Fill,

            #[wrap(Some)]
            set_start_child = &gtk::Box{
                set_vexpand: true,
                set_hexpand: true,
                set_valign: gtk::Align::Fill,
                set_margin_all: 5,
                set_spacing: 5,
                set_css_classes: &["thread_list"],

                #[local_ref]
                thread_list -> gtk::Box {},
            },

            #[wrap(Some)]
            set_end_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
                set_vexpand: true,
                set_hexpand: true,
                set_valign: gtk::Align::Fill,
                set_halign: gtk::Align::Fill,

                // Assistant Parameters
                #[local_ref]
                assistant_parameters -> gtk::Box {},

                // Message bubbles
                #[local_ref]
                message_bubbles -> gtk::Box {},

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
        let database = init.1;

        let threads = {
            let mut database = database.lock().await;
            let mut threads = database
                .get_threads()
                .await
                .expect("Getting all thread should work");

            if threads.is_empty() {
                tracing::info!("No threads were found. Creating new one");
                let thread = database
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
            let mut database = database.lock().await;
            database
                .get_messages(latest_thread_id)
                .await
                .expect("Getting messages should work")
        };

        let message_bubbles = MessageBubbleContainerComponent::builder()
            .launch(messages)
            .detach();

        let local_models = {
            let assistant = assistant.lock().await;
            match assistant.list_models().await {
                Ok(models) => models,
                Err(err) => {
                    tracing::error!("Could not retrieve list of local models because of: {err}");
                    Vec::new()
                }
            }
        };

        let assistant_parameters = AssistantParametersComponent::builder()
            .launch(local_models)
            .forward(sender.input_sender(), |output| match output {
                AssistantParametersOutputMsg::Temperature(value) => {
                    ChatScreenInputMsg::Temperature(value)
                }
                AssistantParametersOutputMsg::TopK(value) => ChatScreenInputMsg::TopK(value),
                AssistantParametersOutputMsg::TopP(value) => ChatScreenInputMsg::TopP(value),
                AssistantParametersOutputMsg::ResetParameters => {
                    ChatScreenInputMsg::ResetParameters
                }
                AssistantParametersOutputMsg::SelectModel(value) => {
                    ChatScreenInputMsg::SelectModel(value)
                }
            });

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

        let model = ChatScreen {
            assistant,
            database,
            current_thread_id: latest_thread_id,
            thread_list,
            assistant_parameters,
            chat_input,
            message_bubbles,
        };

        // Connect chat history notifier to message bubbles
        {
            let database = model.database.lock().await;
            database.notifier.subscribe(
                model.message_bubbles.sender(),
                |notifier_message: DatabaseNotifierMessage| match notifier_message {
                    DatabaseNotifierMessage::NewMessage(message) => {
                        Some(MessageBubbleContainerInputMsg::AddNewMessage(message))
                    }
                    DatabaseNotifierMessage::UpdateMessage(message_update) => Some(
                        MessageBubbleContainerInputMsg::AppendToLastMessage(message_update),
                    ),
                    DatabaseNotifierMessage::GetThreadMessages(messages) => {
                        Some(MessageBubbleContainerInputMsg::RefreshMessages(messages))
                    }
                    _ => None,
                },
            );
        }
        // Connect chat history notifier to thread list
        {
            let database = model.database.lock().await;
            database.notifier.subscribe(
                model.thread_list.sender(),
                |notifier_message: DatabaseNotifierMessage| match notifier_message {
                    DatabaseNotifierMessage::NewThread(thread) => {
                        Some(ThreadListContainerInputMsg::AddThread(thread))
                    }
                    DatabaseNotifierMessage::UpdateThread(thread) => {
                        Some(ThreadListContainerInputMsg::UpdateThread(thread))
                    }
                    _ => None,
                },
            );
        }

        // References used in the view macro
        let assistant_parameters = model.assistant_parameters.widget();
        let thread_list = model.thread_list.widget();
        let message_bubbles = model.message_bubbles.widget();
        let chat_input = model.chat_input.widget();

        let widgets = view_output!();

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
                self.assistant.lock().await.set_temperature(value);
            }
            ChatScreenInputMsg::TopK(value) => {
                self.assistant.lock().await.set_top_k(value);
            }
            ChatScreenInputMsg::TopP(value) => {
                self.assistant.lock().await.set_top_p(value);
            }
            ChatScreenInputMsg::ResetParameters => {
                tracing::info!("Resetting assistant parameters");
                let mut assistant = self.assistant.lock().await;
                assistant.reset_parameters();
            }
            ChatScreenInputMsg::SelectModel(model) => {
                tracing::info!("Pulling model {model}");
                self.disable_inputs();
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
                    let mut database = self.database.lock().await;
                    database
                        .get_messages(thread_id)
                        .await
                        .expect("Getting thread messages should work");
                    self.current_thread_id = thread_id;
                }
            }
            ChatScreenInputMsg::CreateNewThread => {
                tracing::info!("Creating new thread");
                let mut database = self.database.lock().await;
                let thread = database
                    .create_thread("New Thread")
                    .await
                    .expect("Creating new thread should work");
                self.current_thread_id = thread.id;
            }
            ChatScreenInputMsg::DeleteThread(thread_id) => {
                tracing::info!("Deleting thread with id {thread_id}");
                let mut database = self.database.lock().await;
                database
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
                    let mut database = self.database.lock().await;
                    let thread_id = self.current_thread_id;
                    database
                        .create_message(thread_id, message.content, message.role)
                        .await
                        .expect("Message should be created");
                }
                sender
                    .input_sender()
                    .emit(ChatScreenInputMsg::AssistantAnswer);

                self.disable_inputs();
            }
            ChatScreenInputMsg::AssistantAnswer => {
                let thread_id = self.current_thread_id;
                let database = self.database.clone();

                let messages: Vec<Message> = {
                    let mut database = database.lock().await;
                    let messages = database
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

                if messages.len() == 2 {
                    tracing::info!("Generating thread title for thread after first user message");
                    let mut assistant = self.assistant.lock().await;
                    let thread_title = assistant
                        .generate_thread_title(messages[1].clone())
                        .await
                        .unwrap();
                    let mut database = database.lock().await;
                    database
                        .update_thread_title(self.current_thread_id, thread_title)
                        .await
                        .expect("Updating thread title should work");
                }

                let assistant_message_id = {
                    let mut database = database.lock().await;
                    let message = database
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
                                        let mut database = database.lock().await;
                                        database
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
                self.enable_inputs();
            }
            ChatScreenCmdMsg::AnswerEnd => {
                self.enable_inputs();
            }
        }
    }
}
