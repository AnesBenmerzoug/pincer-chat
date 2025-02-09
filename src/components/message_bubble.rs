use anyhow::{anyhow, Result};
use chrono::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::ollama::types::{Message, Role};

#[derive(Debug)]
pub struct MessageBubbleContainerComponent {
    message_bubbles: FactoryVecDeque<MessageBubbleComponent>,
}

#[derive(Debug)]
pub enum MessageBubbleContainerInputMsg {
    AddMessage(Message),
    ReplaceLastMessage(Message),
}

#[relm4::component(pub)]
impl Component for MessageBubbleContainerComponent {
    type Init = ();
    type Input = MessageBubbleContainerInputMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,

            #[local]
            factory_box -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 10,
                set_spacing: 10,
                set_hexpand: false,
                set_vexpand: true,
                // set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,
            },
        },
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let factory_box = gtk::Box::default();

        let message_bubbles = FactoryVecDeque::builder()
            .launch(factory_box.clone())
            .detach();

        let model = MessageBubbleContainerComponent { message_bubbles };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            MessageBubbleContainerInputMsg::AddMessage(message) => {
                let mut guard = self.message_bubbles.guard();
                guard.push_back(message);
            }
            MessageBubbleContainerInputMsg::ReplaceLastMessage(message) => {
                let mut guard = self.message_bubbles.guard();
                guard
                    .back_mut()
                    .expect("There should be at least one previous message")
                    .replace_message(message)
                    .expect("Replacing message should work");
            }
        }
    }
}

#[derive(Debug)]
pub struct MessageBubbleComponent {
    buffer: gtk::TextBuffer,
    role: Role,
    timestamp: String,
}

impl MessageBubbleComponent {
    pub fn replace_message(&mut self, other: Message) -> Result<()> {
        if self.role != other.role {
            return Err(anyhow!("the two message roles should be the same"));
        }
        self.buffer.set_text(&*other.content);
        Ok(())
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for MessageBubbleComponent {
    type Init = Message;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box{
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::Fill,

            gtk::Label {
                set_text: &*self.timestamp,
            },
            gtk::TextView {
                #[watch]
                set_buffer: Some(&self.buffer),
                set_focusable: false,
                set_editable: false,
                set_justification: gtk::Justification::Left,
                set_wrap_mode: gtk::WrapMode::Word,
                add_css_class: match self.role {
                    Role::System => "system_message",
                    Role::User => "user_message",
                    Role::Assistant => "assistant_message",
                    Role::Tool => "tool_message",
                }
            }
        }
    }

    fn init_model(
        message: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        let buffer = gtk::TextBuffer::default();
        buffer.set_text(&*message.content);
        let role = message.role;
        let timestamp = Local::now();
        let timestamp = timestamp.format("%d %B %Y at %R").to_string();
        Self {
            buffer,
            role,
            timestamp,
        }
    }
}
