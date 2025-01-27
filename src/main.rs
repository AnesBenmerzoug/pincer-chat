mod assistant;

use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;

use crate::assistant::types::{ Message, Role };

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";


#[derive(Debug)]
struct App {
    messages: FactoryVecDeque<MessageComponent>,
    user_input: gtk::EntryBuffer,
}

#[derive(Debug)]
enum AppMsg {
    Submit
}

#[derive(Debug)]
struct MessageComponent {
    message: Message
}

#[relm4::factory]
impl FactoryComponent for MessageComponent {
    type Init = Message;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Text {
            set_text: &self.message.content,
            add_css_class: match self.message.role {
                Role::System => "system_message",
                Role::User => "user_message",
                Role::Assistant => "assistant_message",
                Role::Tool => "tool_message",
            }
        }
    }

    fn init_model(message: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { message }
    }
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Chat"),
            set_default_size: (800, 600),
            set_hexpand: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::ScrolledWindow {
                    #[local]
                    factory_box -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 5,
                        set_spacing: 5,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,
                    },
                },

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
                        connect_activate => AppMsg::Submit,
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                    },
                    gtk::Button {
                        set_label: "Send",
                        connect_clicked => AppMsg::Submit,
                    }
                }
            }
        }
    }

    fn init(
        _: (),
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let factory_box = gtk::Box::default();
        
        let messages = FactoryVecDeque::builder()
            .launch(factory_box.clone())
            .forward(sender.input_sender(), |_| AppMsg::Submit);

        let model = App { messages: messages, user_input: gtk::EntryBuffer::default(), };

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Submit => {
                let text = self.user_input.text();
                if !text.is_empty() {
                    let mut guard = self.messages.guard();
                    guard.push_back(Message{content: text.to_string(), role: Role::User});
                    // clearing the entry value clears the entry widget
                    self.user_input.set_text("");
                    // emulate assistant response
                    guard.push_back(Message{content: "I am sorry but I do not know".to_string(), role: Role::Assistant});
                }
            }
        }
    }
}

fn main() {
    let relm = RelmApp::new(APP_ID);
    relm4::set_global_css_from_file("assets/style.css").expect("Expected a stylesheet");
    relm.run::<App>(());
}