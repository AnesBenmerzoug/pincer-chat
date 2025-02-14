use gtk::prelude::*;
use relm4::prelude::*;
use tracing;

use crate::components::chat_input::{ChatInputComponent, ChatInputInputMsg, ChatInputOutputMsg};
use crate::components::message_bubble::{
    MessageBubbleContainerComponent, MessageBubbleContainerInputMsg,
};
use crate::ollama::types::{Message, Role};

#[derive(Debug)]
pub struct ChatPage {
    // Components
    message_bubbles: Controller<MessageBubbleContainerComponent>,
    chat_input: Controller<ChatInputComponent>,
}

#[derive(Debug)]
pub enum ChatPageInputMsg {
    SelectModel(String),
    ModelReady,
    SubmitUserInput(String),
    AssistantAnswerStart,
    AssistantAnswerProgress(Message),
    AssistantAnswerEnd,
}

#[derive(Debug)]
pub enum ChatPageOutputMsg {
    TriggerModelPull(String),
    GetAssistantAnswer(Message),
}

#[relm4::component(pub)]
impl SimpleComponent for ChatPage {
    type Init = ();
    type Input = ChatPageInputMsg;
    type Output = ChatPageOutputMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_css_classes: &["main_container"],

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
                gtk::DropDown::from_strings(&["deepseek-r1:1.5b", "deepseek-r1", "llama3.2:1b", "llama3.2"]) {
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

        let default_model = widgets
            .model_selection_drop_down
            .selected_item()
            .unwrap()
            .downcast::<gtk::StringObject>()
            .unwrap()
            .into();
        sender.input(ChatPageInputMsg::SelectModel(default_model));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ChatPageInputMsg::SelectModel(model) => {
                self.chat_input.sender().emit(ChatInputInputMsg::Disable);
                sender
                    .output_sender()
                    .emit(ChatPageOutputMsg::TriggerModelPull(model));
            }
            ChatPageInputMsg::ModelReady => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
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
                self.chat_input.sender().emit(ChatInputInputMsg::Disable);
            }
            ChatPageInputMsg::AssistantAnswerStart => {
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AddMessage(Message {
                        role: Role::Assistant,
                        content: String::new(),
                    }));
            }
            ChatPageInputMsg::AssistantAnswerProgress(message) => {
                self.message_bubbles
                    .sender()
                    .emit(MessageBubbleContainerInputMsg::AppendToLastMessage(message));
            }
            ChatPageInputMsg::AssistantAnswerEnd => {
                self.chat_input.sender().emit(ChatInputInputMsg::Enable);
            }
        }
    }
}
