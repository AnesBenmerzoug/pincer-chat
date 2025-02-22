use gtk::prelude::*;
use relm4::prelude::*;

use crate::components::assistant_options_dialog::AssistantOptionsDialog;

use super::assistant_options_dialog::AssistantOptionsDialogInputMsg;

#[derive(Debug)]
pub struct ChatInputComponent {
    enabled: bool,
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
    SubmitUserInput(String),
}

impl ChatInputComponent {
    fn disable(&mut self) {
        self.enabled = false;
    }

    fn enable(&mut self) {
        self.enabled = true;
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
            #[watch]
            set_sensitive: model.enabled,

            #[name = "text_input"]
            gtk::Entry {
                set_buffer: &model.user_input,
                #[watch]
                set_tooltip_text: Some("Write a message"),
                #[watch]
                set_placeholder_text: if model.enabled == true { Some("Write a message") } else { Some("Loading ...") },
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                connect_activate => ChatInputInputMsg::Submit,
            },

            #[name = "submit_button"]
            gtk::Button {
                set_label: "Send",
                set_css_classes: &["submit_button"],

                connect_clicked => ChatInputInputMsg::Disable,
            },

            #[name = "option_menu_button"]
            gtk::Button {
                set_icon_name: "open-menu-symbolic",
                set_icon_name: "preferences-system-symbolic",
                set_css_classes: &["option_menu_button"],
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
            enabled: true,
            user_input: gtk::EntryBuffer::default(),
            options_dialog,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            ChatInputInputMsg::Enable => {
                self.enable();
            }
            ChatInputInputMsg::Disable => {
                self.disable();
            }
            ChatInputInputMsg::Submit => {
                let text = self.user_input.text();
                if !text.is_empty() {
                    tracing::info!("Submitting user input {}", text.to_string());
                    sender
                        .output(ChatInputOutputMsg::SubmitUserInput(text.to_string()))
                        .expect("Sending component message should work");
                    tracing::info!("Clearing user input field");
                    self.user_input.set_text("");
                    tracing::info!("Disabling user input temporarily");
                    self.disable();
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
