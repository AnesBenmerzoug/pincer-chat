mod assistant;
mod components;
mod screens;

use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use relm4::RelmRemoveAllExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing;

use crate::assistant::{ollama::types::Message, Assistant, AssistantParameters};
use crate::screens::{
    chat::{ChatPage, ChatPageInputMsg, ChatPageOutputMsg},
    startup::{StartupPage, StartupPageOutputMsg},
};

const APP_ID: &str = "org.relm4.RustyLocalAIAssistant";

#[derive(Debug)]
struct App {
    assistant: Arc<Mutex<Assistant>>,
    screen: Option<AppScreen>,
}

#[derive(Debug)]
enum AppScreen {
    StartUp(AsyncController<StartupPage>),
    Chat(AsyncController<ChatPage>),
}

#[derive(Debug)]
enum AppMsg {
    ShowStartUpScreen,
    ShowChatScreen,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_title: Some("Chat"),
            set_default_size: (800, 600),
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::Fill,

            #[name = "container"]
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,
                set_width_request: 800,
                set_height_request: 600,
            }
        },
    }

    async fn init(
        _: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let assistant = Assistant::new();
        /*
        let models = match assistant.list_models().await {
            Ok(models) => models,
            Err(err) => {
                tracing::error!("Could not retrieve list of local models because of: {err}");
                panic!("Could not retrieve list of local models");
            }
        };

        let mut model = String::from("llama3.2:1b");
        if models.len() == 0 {
            tracing::info!("No local model found. Using {model} as default model");
        } else {
            tracing::info!(
                "Found {} local models. Using {} as default model",
                models.len(),
                models[0]
            );
            model = models[0].clone();
        }

        assistant.set_model(model.clone());
        {
            tracing::info!("Pulling {model}");
            let (response_sender, _) = mpsc::channel();
            match assistant.pull_model(model, response_sender).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        */

        let mut model = App {
            assistant: Arc::new(Mutex::new(assistant)),
            screen: None,
        };

        let mut widgets = view_output! {};

        model
            .update_with_view(&mut widgets, AppMsg::ShowStartUpScreen, sender, &root)
            .await;

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        widgets.container.remove_all();
        match message {
            AppMsg::ShowStartUpScreen => {
                tracing::info!("Showing startup screen");
                let assistant = self.assistant.clone();
                let controller = StartupPage::builder().launch(assistant).forward(
                    sender.input_sender(),
                    |output| match output {
                        StartupPageOutputMsg::End => AppMsg::ShowChatScreen,
                    },
                );
                widgets.container.append(controller.widget());
                self.screen = Some(AppScreen::StartUp(controller));
            }
            AppMsg::ShowChatScreen => {
                let assistant = self.assistant.clone();
                let controller = ChatPage::builder().launch(assistant).detach();
                widgets.container.append(controller.widget());
                self.screen = Some(AppScreen::Chat(controller));
            }
        }
    }
}

fn load_css(settings: &gtk::Settings) {
    let theme_name = settings
        .gtk_theme_name()
        .expect("Could not get theme name.");

    // Load common style sheet
    relm4::set_global_css_from_file("assets/common.css").expect("Expected a stylesheet");

    // Load mode-specific style sheet
    if theme_name.to_lowercase().contains("dark") || settings.is_gtk_application_prefer_dark_theme()
    {
        relm4::set_global_css_from_file("assets/dark.css").expect("Expected a stylesheet");
    } else {
        relm4::set_global_css_from_file("assets/light.css").expect("Expected a stylesheet");
    }
}

fn main() {
    // Show traces to find potential performance bottlenecks, for example
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::TRACE)
        .init();

    tracing::info!("Starting application!");
    let relm_app = RelmApp::new(APP_ID);

    let settings = gtk::Settings::default().expect("Accessing settings should work");
    settings.connect_gtk_application_prefer_dark_theme_notify(load_css);
    settings.connect_gtk_theme_name_notify(load_css);
    load_css(&settings);

    relm_app.run_async::<App>(());
}
