use gtk::glib;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::assistant::AssistantParameters;

#[derive(Debug)]
pub struct AssistantOptionsDialog {
    options: AssistantParameters,
    visible: bool,
}

#[derive(Debug)]
pub enum AssistantOptionsDialogInputMsg {
    Show,
    ResetOptions,
    SendOptions,
    CancelOptions,
    SelectedModel(String),
    Temperature(f64),
    TopK(u64),
    TopP(f64),
}

#[derive(Debug)]
pub enum AssistantOptionsDialogOutputMsg {
    SendOptions(AssistantParameters),
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for ParameterSpinButton {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 5,
            set_spacing: 5,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::Start,
        }
    }
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
                        let selected_model = model_drop_down
                            .selected_item()
                            .expect("Getting selected item from dropdown should work")
                            .downcast::<gtk::StringObject>()
                            .expect("Conversion of gtk StringObject to String should work")
                            .into();
                        sender.input(AssistantOptionsDialogInputMsg::SelectedModel(selected_model));
                    },
                },
            },

            // Temperature
            #[template]
            ParameterSpinButton {
                gtk::Label {
                    set_label: "Temperature",
                },
                gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                    #[watch]
                    set_value: model.options.temperature,

                    connect_value_changed[sender] => move |btn| {
                        let value = btn.value();
                        sender.input(AssistantOptionsDialogInputMsg::Temperature(value));
                    },
                },
            },
            // Top-K
            #[template]
            ParameterSpinButton {
                gtk::Label {
                    set_label: "Top-K",
                },
                gtk::SpinButton::with_range(0.0, 100.0, 1.0) {
                    #[watch]
                    set_value: model.options.top_k as f64,

                    connect_value_changed[sender] => move |btn| {
                        let value = btn.value() as u64;
                        sender.input(AssistantOptionsDialogInputMsg::TopK(value));
                    },
                },
            },
            // Top-P
            #[template]
            ParameterSpinButton {
                gtk::Label {
                    set_label: "Top-P",
                },
                gtk::SpinButton::with_range(0.0, 1.0, 0.1) {
                    #[watch]
                    set_value: model.options.top_p,

                    connect_value_changed[sender] => move |btn| {
                        let value = btn.value();
                        sender.input(AssistantOptionsDialogInputMsg::TopP(value));
                    },
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Button {
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_icon_name: "mail-send-symbolic",
                    set_tooltip_text: Some("Apply options"),
                    set_css_classes: &["send_button"],
                    connect_clicked => AssistantOptionsDialogInputMsg::SendOptions,
                },
                gtk::Button {
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_icon_name: "window-close-symbolic",
                    set_tooltip_text: Some("Cancel option changes"),
                    set_css_classes: &["cancel_button"],
                    connect_clicked => AssistantOptionsDialogInputMsg::CancelOptions,
                },
                gtk::Button {
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_icon_name: "edit-undo-symbolic",
                    set_tooltip_text: Some("Restore default options"),
                    set_css_classes: &["reset_button"],
                    connect_clicked => AssistantOptionsDialogInputMsg::ResetOptions,
                },
            },

            connect_close_request[sender] => move |_| {
                sender.input(AssistantOptionsDialogInputMsg::SendOptions);
                glib::Propagation::Stop
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = AssistantOptionsDialog {
            options: AssistantParameters::default(),
            visible: false,
        };
        let widgets = view_output!();
        let default_model: String = widgets
            .model_selection_drop_down
            .selected_item()
            .expect("Getting selected item from dropdown should work")
            .downcast::<gtk::StringObject>()
            .expect("Conversion of gtk StringObject to String should work")
            .into();
        model.options.model = Some(default_model);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AssistantOptionsDialogInputMsg::Show => self.visible = true,
            AssistantOptionsDialogInputMsg::SelectedModel(value) => {
                self.options.model = Some(value)
            }
            AssistantOptionsDialogInputMsg::Temperature(value) => self.options.temperature = value,
            AssistantOptionsDialogInputMsg::TopK(value) => self.options.top_k = value,
            AssistantOptionsDialogInputMsg::TopP(value) => self.options.top_p = value,
            AssistantOptionsDialogInputMsg::ResetOptions => {
                self.options = AssistantParameters::default()
            }
            AssistantOptionsDialogInputMsg::CancelOptions => self.visible = false,
            AssistantOptionsDialogInputMsg::SendOptions => {
                sender
                    .output_sender()
                    .emit(AssistantOptionsDialogOutputMsg::SendOptions(
                        self.options.clone(),
                    ));
                self.visible = false;
            }
        }
    }
}
