use anyhow::{anyhow, Result};
use gtk::prelude::*;
use relm4::prelude::*;

use crate::ollama::types::{Message, Role};

#[derive(Debug)]
pub struct MessageComponent {
    message: Message,
    buffer: gtk::TextBuffer,
}

impl MessageComponent {
    pub fn replace_message(&mut self, other: Message) -> Result<()> {
        if self.message.role != other.role {
            return Err(anyhow!("the two message roles should be the same"));
        }
        self.message = other;
        self.buffer.set_text(&*self.message.content);
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
        gtk::TextView {
            #[watch]
            set_buffer: Some(&self.buffer),
            set_focusable: false,
            set_editable: false,
            set_justification: match self.message.role {
                Role::User => gtk::Justification::Right,
                _ => gtk::Justification::Left,
            },
            set_wrap_mode: gtk::WrapMode::Word,
            add_css_class: match self.message.role {
                Role::System => "system_message",
                Role::User => "user_message",
                Role::Assistant => "assistant_message",
                Role::Tool => "tool_message",
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
        Self { message, buffer }
    }
}
