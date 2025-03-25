use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct AssistantParametersComponent {
    model: String,
    generation_parameters: GenerationParameters,
}

#[derive(Debug)]
pub struct GenerationParameters {
    pub temperature: f64,
    pub top_k: u64,
    pub top_p: f64,
}

impl Default for GenerationParameters {
    fn default() -> Self {
        Self {
            temperature: 0.5,
            top_k: 40,
            top_p: 0.9,
        }
    }
}

#[derive(Debug)]
pub enum AssistantParametersInputMsg {
    SelectModel(String),
    Temperature(f64),
    TopK(u64),
    TopP(f64),
    ResetParameters,
}

#[derive(Debug)]
pub enum AssistantParametersOutputMsg {
    SelectModel(String),
    Temperature(f64),
    TopK(u64),
    TopP(f64),
    ResetParameters,
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for ParameterSpinButton {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::Center,
        }
    }
}

#[relm4::component(pub)]
impl Component for AssistantParametersComponent {
    type Init = Vec<String>;
    type Input = AssistantParametersInputMsg;
    type Output = AssistantParametersOutputMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_hexpand: true,
            set_halign: gtk::Align::Fill,

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
                gtk::DropDown {
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_css_classes: &["dropdown", "model_dropdown"],

                    connect_selected_notify[sender] => move |model_drop_down| {
                        sender.input(AssistantParametersInputMsg::SelectModel(
                            model_drop_down
                            .selected_item()
                            .expect("Getting selected item from dropdown should work")
                            .downcast::<gtk::StringObject>()
                            .expect("Conversion of gtk StringObject to String should work")
                            .into()))
                    },
                },

                gtk::MenuButton {
                    set_icon_name: "preferences-system-symbolic",
                    set_direction: gtk::ArrowType::Down,
                    set_css_classes: &["button", "options_menu_button"],

                    #[wrap(Some)]
                    set_popover: popover = &gtk::Popover {
                        set_position: gtk::PositionType::Bottom,
                        set_halign: gtk::Align::Fill,
                        set_hexpand: true,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,

                            // Temperature
                            #[template]
                            ParameterSpinButton {
                                gtk::Label {
                                    set_label: "Temperature",
                                    set_justify: gtk::Justification::Left,
                                },
                                gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.1) {
                                    set_halign: gtk::Align::Fill,
                                    set_hexpand: true,
                                    #[watch]
                                    set_value: model.generation_parameters.temperature,
                                    set_draw_value: true,

                                    connect_value_changed[sender] => move |btn| {
                                        let value = btn.value();
                                        sender.input(AssistantParametersInputMsg::Temperature(value));
                                    },
                                },
                            },

                            // Top-K
                            #[template]
                            ParameterSpinButton {
                                gtk::Label {
                                    set_label: "Top-K",
                                    set_halign: gtk::Align::Fill,
                                    set_justify: gtk::Justification::Left,
                                },
                                gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0) {
                                    set_halign: gtk::Align::Fill,
                                    #[watch]
                                    set_value: model.generation_parameters.top_k as f64,
                                    set_draw_value: true,

                                    connect_value_changed[sender] => move |btn| {
                                        let value = btn.value() as u64;
                                        sender.input(AssistantParametersInputMsg::TopK(value));
                                    },
                                },
                            },

                            // Top-P
                            #[template]
                            ParameterSpinButton {
                                gtk::Label {
                                    set_label: "Top-P",
                                    set_halign: gtk::Align::Fill,
                                    set_justify: gtk::Justification::Left,
                                },
                                gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.1) {
                                    set_halign: gtk::Align::Fill,
                                    #[watch]
                                    set_value: model.generation_parameters.top_p,
                                    set_draw_value: true,

                                    connect_value_changed[sender] => move |btn| {
                                        let value = btn.value();
                                        sender.input(AssistantParametersInputMsg::TopP(value));
                                    },
                                },
                            },

                            gtk::Button {
                                set_hexpand: true,
                                set_halign: gtk::Align::Fill,
                                set_icon_name: "edit-undo-symbolic",
                                set_tooltip_text: Some("Restore default options"),
                                set_css_classes: &["button", "reset_options_button"],
                                connect_clicked => AssistantParametersInputMsg::ResetParameters,
                            },
                        },
                    },
                },
            },
        },
    }

    fn init(
        models: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AssistantParametersComponent {
            model: models[0].clone(),
            generation_parameters: GenerationParameters::default(),
        };

        let widgets = view_output!();

        let model_list = gtk::StringList::default();
        for model_name in models {
            model_list.append(&model_name);
        }
        widgets
            .model_selection_drop_down
            .set_model(Some(&model_list));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        match message {
            AssistantParametersInputMsg::Temperature(value) => {
                self.generation_parameters.temperature = value;
                sender
                    .output_sender()
                    .emit(AssistantParametersOutputMsg::Temperature(value));
            }
            AssistantParametersInputMsg::TopK(value) => {
                self.generation_parameters.top_k = value;
                sender
                    .output_sender()
                    .emit(AssistantParametersOutputMsg::TopK(value));
            }
            AssistantParametersInputMsg::TopP(value) => {
                self.generation_parameters.top_p = value;
                sender
                    .output_sender()
                    .emit(AssistantParametersOutputMsg::TopP(value));
            }
            AssistantParametersInputMsg::ResetParameters => {
                self.generation_parameters = GenerationParameters::default();
                sender
                    .output_sender()
                    .emit(AssistantParametersOutputMsg::ResetParameters);
            }
            AssistantParametersInputMsg::SelectModel(model) => {
                self.model = model.clone();
                sender
                    .output_sender()
                    .emit(AssistantParametersOutputMsg::SelectModel(model));
            }
        }
    }
}
