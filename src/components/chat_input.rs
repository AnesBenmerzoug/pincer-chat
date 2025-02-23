use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ChatInputComponent {
    enabled: bool,
    user_input: gtk::EntryBuffer,
}

#[derive(Debug)]
pub enum ChatInputInputMsg {
    Enable,
    Disable,
    Submit,
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
                set_tooltip_text: Some("Submit message"),
                set_icon_name: "document-send-symbolic",
                set_css_classes: &["submit_button"],

                connect_clicked => ChatInputInputMsg::Disable,
            },
        },
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ChatInputComponent {
            enabled: true,
            user_input: gtk::EntryBuffer::default(),
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
        }
    }
}
