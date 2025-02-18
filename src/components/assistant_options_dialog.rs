use gtk::glib;
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct AssistantOptionsDialog {
    visible: bool,
}

#[derive(Debug)]
pub enum AssistantOptionsDialogInputMsg {
    Show,
    Hide,
    SelectedModel(String),
}

#[derive(Debug)]
pub enum AssistantOptionsDialogOutputMsg {
    SelectedModel(String),
}

#[relm4::component(pub)]
impl SimpleComponent for AssistantOptionsDialog {
    type Init = ();
    type Input = AssistantOptionsDialogInputMsg;
    type Output = AssistantOptionsDialogOutputMsg;

    view! {
        dialog = gtk::Dialog {
            set_title: Some("Assistant Options"),
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_css_classes: &["assistant_options_dialog"],
            #[watch]
            set_visible: model.visible,
            set_modal: true,

            // Model Selection
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Start,

                gtk::Label {
                    set_label: "Model",
                },
                #[name = "model_selection_drop_down"]
                gtk::DropDown::from_strings(&["deepseek-r1:1.5b", "deepseek-r1", "llama3.2:1b", "llama3.2"]) {
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    connect_selected_notify[sender] => move |model_drop_down| {
                        sender.input(AssistantOptionsDialogInputMsg::SelectedModel(
                            model_drop_down
                            .selected_item()
                            .expect("Getting selected item from dropdown should work")
                            .downcast::<gtk::StringObject>()
                            .expect("Conversion of gtk StringObject to String should work")
                            .into()))
                    },
                },
            },

            // Temperature
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Start,

                gtk::Label {
                    set_label: "Temperature",
                },
                gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                    set_value: 0.5,
                },
            },
            // Top-K
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Start,

                gtk::Label {
                    set_label: "Top-K",
                },
                gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                    set_value: 0.5,
                },
            },
            // Top-P
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Start,

                gtk::Label {
                    set_label: "Top-P",
                },
                gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                    set_value: 0.5,
                },
            },
            // Stop-Word
            // Seed

            connect_close_request[sender] => move |_| {
                sender.input(AssistantOptionsDialogInputMsg::Hide);
                glib::Propagation::Stop
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AssistantOptionsDialog { visible: false };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AssistantOptionsDialogInputMsg::Show => self.visible = true,
            AssistantOptionsDialogInputMsg::Hide => self.visible = false,
            AssistantOptionsDialogInputMsg::SelectedModel(model) => {
                sender
                    .output(AssistantOptionsDialogOutputMsg::SelectedModel(model))
                    .expect("Message to be sent over channel");
            }
        }
    }
}
