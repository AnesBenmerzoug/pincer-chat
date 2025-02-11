use gtk::prelude::*;
use relm4::prelude::*;

use crate::components::assistant_options_dialog::AssistantOptionsDialog;

use super::assistant_options_dialog::AssistantOptionsDialogInputMsg;

#[derive(Debug)]
pub struct ChatInputComponent {
    user_input: gtk::EntryBuffer,
    options_dialog: Controller<AssistantOptionsDialog>,
}

#[derive(Debug)]
pub enum ChatInputInputMsg {
    Enable,
    Disable,
    Submit,
    ShowOptionsDialog,
}

#[derive(Debug)]
pub enum ChatInputOutputMsg {
    UserMessage(String),
}

impl ChatInputComponent {
    fn disable(&self, root: &<Self as Component>::Root) {
        root.set_sensitive(false);
    }

    fn enable(&self, root: &<Self as Component>::Root) {
        root.set_sensitive(true);
    }
}

#[relm4::component(pub)]
impl Component for ChatInputComponent {
    type Init = ();
    type Input = ChatInputInputMsg;
    type Output = ChatInputOutputMsg;
    type CommandOutput = ();

    view! {
        #[root]
        chat_input_container = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 5,
            set_spacing: 5,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::End,

            #[name = "text_input"]
            gtk::Entry {
                set_buffer: &model.user_input,
                set_tooltip_text: Some("Send a message"),
                set_placeholder_text: Some("Send a message"),
                set_vexpand: true,
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                connect_activate => ChatInputInputMsg::Submit,
            },

            #[name = "submit_button"]
            gtk::Button {
                set_label: "Send",
                set_vexpand: true,
                set_css_classes: &["submit_button"],
                set_sensitive: true,

                connect_clicked => ChatInputInputMsg::Disable,
            },

            #[name = "option_menu_button"]
            gtk::Button {
                set_icon_name: "open-menu-symbolic",
                set_vexpand: true,
                set_icon_name: "preferences-system-symbolic",
                set_css_classes: &["option_menu_button"],
                set_sensitive: true,
                connect_clicked => ChatInputInputMsg::ShowOptionsDialog,
            }
        },
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let options_dialog = AssistantOptionsDialog::builder()
            .transient_for(&root)
            .launch(())
            .detach();

        let model = ChatInputComponent {
            user_input: gtk::EntryBuffer::default(),
            options_dialog,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            ChatInputInputMsg::Enable => {
                self.enable(root);
            }
            ChatInputInputMsg::Disable => {
                self.disable(root);
            }
            ChatInputInputMsg::Submit => {
                let text = self.user_input.text();
                if !text.is_empty() {
                    tracing::info!("Submitting user input {}", text.to_string());
                    sender
                        .output(ChatInputOutputMsg::UserMessage(text.to_string()))
                        .expect("Sending componet message should work");
                    tracing::info!("Clearing user input field");
                    self.user_input.set_text("");
                    tracing::info!("Disabling user input temporarily");
                    self.disable(root);
                };
            }
            ChatInputInputMsg::ShowOptionsDialog => {
                self.options_dialog
                    .sender()
                    .emit(AssistantOptionsDialogInputMsg::Show);
            }
        }
    }
}
