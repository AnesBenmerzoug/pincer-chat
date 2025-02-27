use anyhow::{anyhow, Result};
use chrono::prelude::*;
use gtk::prelude::*;
use relm4::{gtk::subclass::adjustment, prelude::*};

use crate::assistant::ollama::types::{Message, Role};

#[derive(Debug)]
pub struct MessageBubbleContainerComponent {
    message_bubbles: FactoryVecDeque<MessageBubbleComponent>,
}

#[derive(Debug)]
pub enum MessageBubbleContainerInputMsg {
    AddMessage(Message),
    AppendToLastMessage(Message),
}

#[relm4::component(pub)]
impl Component for MessageBubbleContainerComponent {
    type Init = ();
    type Input = MessageBubbleContainerInputMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box{
            set_vexpand: true,
            set_valign: gtk::Align::Fill,

            #[name = "scrolled_window"]
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,

                #[local]
                factory_box -> gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 10,
                    set_spacing: 10,
                },
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

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            MessageBubbleContainerInputMsg::AddMessage(message) => {
                let mut guard = self.message_bubbles.guard();
                guard.push_back(message);
                let adjustment = widgets.scrolled_window.vadjustment();
                adjustment.set_value(adjustment.upper());
                widgets.scrolled_window.set_vadjustment(Some(&adjustment));
            }
            MessageBubbleContainerInputMsg::AppendToLastMessage(message) => {
                let mut guard = self.message_bubbles.guard();
                guard
                    .back_mut()
                    .expect("There should be at least one previous message")
                    .append_to_message(message)
                    .expect("Replacing message should work");

                let adjustment = widgets.scrolled_window.vadjustment();
                adjustment.set_value(adjustment.upper());
                widgets.scrolled_window.set_vadjustment(Some(&adjustment));
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
    pub fn new(message: Message) -> Self {
        let buffer = gtk::TextBuffer::builder().text(&*message.content).build();
        let timestamp = Local::now().format("%d %B %Y at %R").to_string();
        let role = message.role;
        Self {
            buffer,
            role,
            timestamp,
        }
    }

    pub fn append_to_message(&mut self, other: Message) -> Result<()> {
        if self.role != other.role {
            return Err(anyhow!("the two message roles should be the same"));
        }
        self.buffer.insert_at_cursor(&*other.content);
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
                set_height_request: 40,

                #[watch]
                set_buffer: Some(&self.buffer),
                set_focusable: false,
                set_editable: false,
                set_justification: gtk::Justification::Left,
                set_wrap_mode: gtk::WrapMode::Word,
                set_css_classes: &["message_bubble"],
                add_css_class: match self.role {
                    Role::System => "system_message",
                    Role::User => "user_message",
                    Role::Assistant => "assistant_message",
                    Role::Tool => "tool_message",
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self::new(init)
    }
}
