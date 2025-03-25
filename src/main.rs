mod assets;
mod assistant;
mod components;
mod screens;

use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender};
use relm4::prelude::*;
use relm4::RelmRemoveAllExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use assistant::{database::Database, Assistant};
use screens::{
    chat::ChatScreen,
    startup::{StartupScreen, StartupScreenOutputMsg},
};

const APP_ID: &str = "org.relm4.PincerChat";

#[derive(Debug)]
struct App {
    assistant: Arc<Mutex<Assistant>>,
    chat_history: Arc<Mutex<Database>>,
    screen: Option<AppScreen>,
}

#[derive(Debug)]
enum AppScreen {
    StartUp(#[allow(dead_code)] AsyncController<StartupScreen>),
    Chat(#[allow(dead_code)] AsyncController<ChatScreen>),
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
                set_css_classes: &["main_container"],
            }
        },
    }

    async fn init(
        _: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let assistant = Assistant::new().await;
        let chat_history = Database::new(None)
            .await
            .expect("Database connection should work");

        let mut model = App {
            assistant: Arc::new(Mutex::new(assistant)),
            chat_history: Arc::new(Mutex::new(chat_history)),
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
                let chat_history = self.chat_history.clone();
                let controller = StartupScreen::builder()
                    .launch((assistant, chat_history))
                    .forward(sender.input_sender(), |output| match output {
                        StartupScreenOutputMsg::End => AppMsg::ShowChatScreen,
                    });
                widgets.container.append(controller.widget());
                self.screen = Some(AppScreen::StartUp(controller));
            }
            AppMsg::ShowChatScreen => {
                let assistant = self.assistant.clone();
                let chat_history = self.chat_history.clone();
                let controller = ChatScreen::builder()
                    .launch((assistant, chat_history))
                    .detach();
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
