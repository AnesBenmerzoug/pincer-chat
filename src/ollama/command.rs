use crate::ollama::{client::OllamaClient, types::Message};
use gtk::prelude::*;
use relm4::{
    component::{AsyncComponentParts, AsyncComponentSender, SimpleAsyncComponent},
    gtk,
    loading_widgets::LoadingWidgets,
    view, RelmApp, RelmWidgetExt,
};

#[derive(Debug)]
pub struct OllamaComponent {
    ollama_client: OllamaClient,
}

#[derive(Debug)]
pub enum OllamaMsg {
    Chat(Message),
}

impl SimpleAsyncComponent for OllamaComponent {
    type Init = String;
    type Input = OllamaMsg;
    type Output = ();
    type Root = ();
    type Widgets = ();

    fn init_root() -> Self::Root {}

    async fn init(
        model: Self::Init,
        _: Self::Root,
        _: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let ollama_client = OllamaClient::new(model.to_string());

        let model = OllamaComponent { ollama_client };

        AsyncComponentParts { model, widgets: () }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            OllamaMsg::Chat(message) => {
                //self.ollama_client.generate_answer();
            }
        }
    }
}
