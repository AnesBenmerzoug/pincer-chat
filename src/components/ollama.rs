use futures::FutureExt;

use relm4::{Component, ComponentParts, ComponentSender};

use crate::ollama::{
    client::OllamaClient,
    types::{Message, Role},
};

#[derive(Debug)]
pub struct OllamaComponent {
    ollama_client: OllamaClient,
    messages: Vec<Message>,
}

#[derive(Debug)]
pub enum OllamaInputMsg {
    Chat(Message),
}

#[derive(Debug)]
pub enum OllamaOutputMsg {
    Answer(Message),
}

#[derive(Debug)]
pub enum OllamaCmdMsg {
    Answer(String),
}

impl Component for OllamaComponent {
    type Init = ();
    type Input = OllamaInputMsg;
    type Output = OllamaOutputMsg;
    type Root = ();
    type Widgets = ();
    type CommandOutput = OllamaCmdMsg;

    fn init_root() -> Self::Root {}

    fn init(_: Self::Init, _: Self::Root, _: ComponentSender<Self>) -> ComponentParts<Self> {
        let ollama_client = OllamaClient::new();

        let model = OllamaComponent {
            ollama_client,
            messages: Vec::new(),
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _: &Self::Root) {
        println!("Ollama component update");
        match msg {
            OllamaInputMsg::Chat(message) => {
                sender.command(|out, shutdown| {
                    shutdown
                        .register(async move {
                            //self.ollama_client.generate_answer();
                            println!("received message: {:?}", message);
                            out.send(OllamaCmdMsg::Answer("42".into())).unwrap();
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                })
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        println!("Ollama component update_");
        sender
            .output(OllamaOutputMsg::Answer(Message {
                content: "I am sorry but I do not know".to_string(),
                role: Role::Assistant,
            }))
            .unwrap();
    }
}
