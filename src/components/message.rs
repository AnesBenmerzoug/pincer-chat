use anyhow::{anyhow, Result};
use chrono::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::ollama::types::{Message, Role};

#[derive(Debug)]
pub struct MessageComponent {
    buffer: gtk::TextBuffer,
    role: Role,
    timestamp: String,
}

impl MessageComponent {
    pub fn replace_message(&mut self, other: Message) -> Result<()> {
        if self.role != other.role {
            return Err(anyhow!("the two message roles should be the same"));
        }
        self.buffer.set_text(&*other.content);
        Ok(())
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for MessageComponent {
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
                set_justification: match self.role {
                    Role::User => gtk::Justification::Right,
                    _ => gtk::Justification::Left,
                },
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
