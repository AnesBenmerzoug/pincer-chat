use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct ChatInputComponent {
    pub user_input: gtk::EntryBuffer,
}

#[derive(Debug)]
pub enum InputMsg {
    Ready,
    Submit,
}

#[derive(Debug)]
pub enum OutputMsg {
    UserMessage(String),
}

#[relm4::component(pub)]
impl Component for ChatInputComponent {
    type Init = ();
    type Input = InputMsg;
    type Output = OutputMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 5,
            set_spacing: 5,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::End,

            gtk::Entry {
                set_buffer: &model.user_input,
                set_tooltip_text: Some("Send a message"),
                set_placeholder_text: Some("Send a message"),
                set_vexpand: true,
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                connect_activate => InputMsg::Submit,
            },
            gtk::Button {
                set_label: "Send",
                set_vexpand: true,
                set_css_classes: &["submit_button"],

                connect_clicked => InputMsg::Submit,
            },
            /*
            gtk::MenuButton {
                set_vexpand: true,
                set_halign: gtk::Align::Start,
                set_direction: gtk::ArrowType::Up,

                #[watch]
                set_can_target: model.enabled,

                #[wrap(Some)]
                set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {
                    add_child: (&popover_child, "my_widget"),
                }
            }
            */
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ChatInputComponent {
            user_input: gtk::EntryBuffer::default(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            InputMsg::Ready => {
                let _ = Self::enable(root);
            }
            InputMsg::Submit => {
                let text = self.user_input.text();
                if !text.is_empty() {
                    tracing::info!("Submitting user input {}", text.to_string());
                    sender
                        .output(OutputMsg::UserMessage(text.to_string()))
                        .expect("Sending componet message should work");
                    tracing::info!("Clearing user input field");
                    self.user_input.set_text("");
                    tracing::info!("Disabling user input temporarily");
                    let _ = Self::disable(root);
                };
            }
        }
    }
}

impl ChatInputComponent {
    #[must_use]
    fn disable(root: &<Self as Component>::Root) {
        root.set_can_focus(false);
        root.set_can_target(false);
        root.set_child_visible(false);
    }

    #[must_use]
    fn enable(root: &<Self as Component>::Root) {
        root.set_can_focus(true);
        root.set_can_target(true);
        root.set_child_visible(true);
    }
}
