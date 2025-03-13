use anyhow::{anyhow, Result};
use chrono::prelude::*;
use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::factory::{AsyncFactoryComponent, AsyncFactoryVecDeque};
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::view;
use std::future::Future;

use crate::assistant::database::models::Message;
use crate::assistant::ollama::types::Role;

#[derive(Debug)]
pub struct MessageBubbleContainerComponent {
    message_bubbles: AsyncFactoryVecDeque<MessageBubbleComponent>,
}

#[derive(Debug)]
pub enum MessageBubbleContainerInputMsg {
    RefreshMessages(Vec<Message>),
    AddNewMessage(Message),
    AppendToLastMessage(String),
}

#[relm4::component(async, pub)]
impl AsyncComponent for MessageBubbleContainerComponent {
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

    async fn init(
        _: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let factory_box = gtk::Box::default();

        let message_bubbles = AsyncFactoryVecDeque::builder()
            .launch(factory_box.clone())
            .detach();

        let model = MessageBubbleContainerComponent { message_bubbles };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            MessageBubbleContainerInputMsg::RefreshMessages(messages) => {
                let mut guard = self.message_bubbles.guard();
                guard.clear();
                let _ = messages
                    .into_iter()
                    .map(|message| guard.push_back(message))
                    .collect::<Vec<_>>();

                let adjustment = widgets.scrolled_window.vadjustment();
                adjustment.set_value(adjustment.upper() - adjustment.page_size());
                widgets.scrolled_window.set_vadjustment(Some(&adjustment));
            }
            MessageBubbleContainerInputMsg::AddNewMessage(message) => {
                let mut guard = self.message_bubbles.guard();
                guard.push_back(message);

                let adjustment = widgets.scrolled_window.vadjustment();
                adjustment.set_value(adjustment.upper() - adjustment.page_size());
                widgets.scrolled_window.set_vadjustment(Some(&adjustment));
            }
            MessageBubbleContainerInputMsg::AppendToLastMessage(message) => {
                let mut guard = self.message_bubbles.guard();
                guard
                    .back_mut()
                    .expect("There should be at least one previous message")
                    .append_to_message(message)
                    .await
                    .expect("Appending to message should work");

                let adjustment = widgets.scrolled_window.vadjustment();
                adjustment.set_value(adjustment.upper() - adjustment.page_size());
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
    pub async fn new(message: Message) -> Self {
        let buffer = gtk::TextBuffer::builder().text(&*message.content).build();
        let timestamp = Local::now().format("%d %B %Y at %R").to_string();
        let role =
            Role::try_from(message.role).expect("Converting role from string to enum should work");
        Self {
            buffer,
            role,
            timestamp,
        }
    }

    pub async fn append_to_message(&mut self, content: String) -> Result<()> {
        self.buffer.insert_at_cursor(&*content);
        Ok(())
    }
}

#[relm4::factory(async, pub)]
impl AsyncFactoryComponent for MessageBubbleComponent {
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

                //#[watch]
                set_buffer: Some(&self.buffer),
                set_focusable: false,
                set_editable: false,
                set_overwrite: true,
                set_justification: gtk::Justification::Left,
                set_wrap_mode: gtk::WrapMode::WordChar,
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

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local]
            root {
                #[name = "placeholder"]
                gtk::Box{
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 5,
                    set_spacing: 5,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Fill,

                    gtk::Spinner {
                        set_spinning: true,
                    }
                }
            }
        }
        Some(LoadingWidgets::new(root, placeholder))
    }

    fn init_model(
        init: Self::Init,
        _: &DynamicIndex,
        _: AsyncFactorySender<Self>,
    ) -> impl Future<Output = Self> {
        Self::new(init)
    }
}
