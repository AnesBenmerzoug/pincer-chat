use std::str::FromStr;

use mistralrs::{GgufModelBuilder, Model, TextMessageRole, TextMessages, TokenSource};

use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};
use iced::futures::channel::mpsc;
use iced::stream;

pub struct Assistant {
    model_repo_id: String,
    model_file: String,
    tokenizer_repo_id: String,
    model: Model,
}

#[derive(Debug, Clone)]
pub enum Event {
    Loaded(mpsc::Sender<String>),
    AnswerGenerated(String),
}

#[derive(Debug)]
enum State {
    Loading,
    Ready(mpsc::Receiver<String>),
}

pub fn start_assistant(
    model_repo_id: String,
    model_file: String,
    tokenizer_repo_id: String,
) -> impl Stream<Item = Event> {
    stream::channel(100, |mut output| async move {
        let mut state = State::Loading;

        let assistant: Assistant =
            Assistant::new(model_repo_id, model_file, tokenizer_repo_id).await;

        loop {
            match &mut state {
                State::Loading => {
                    let (sender, receiver) = mpsc::channel(1);
                    output.send(Event::Loaded(sender)).await.unwrap();
                    state = State::Ready(receiver);
                }
                State::Ready(receiver) => {
                    futures::select! {
                            input_message = receiver.select_next_some() => {
                                println!("Received input message: {}", input_message);
                                println!("Generating answer");
                                let generated_answer = assistant.generate_answer(input_message).await;
                                println!("Generated answer: {}", generated_answer);
                                output.send(Event::AnswerGenerated(generated_answer)).await.unwrap();
                            }
                    }
                }
            }
        }
    })
}

impl Assistant {
    pub async fn new(model_repo_id: String, model_file: String, tokenizer_repo_id: String) -> Self {
        let model = match GgufModelBuilder::new(model_repo_id.clone(), vec![model_file.clone()])
            .with_tok_model_id(tokenizer_repo_id.clone())
            .with_logging()
            .with_token_source(TokenSource::from_str("env").unwrap())
            .build()
            .await
        {
            Ok(model) => model,
            Err(e) => panic!("Failed loading model {}", e),
        };
        Self {
            model_repo_id,
            model_file,
            tokenizer_repo_id,
            model,
        }
    }

    pub async fn generate_answer(&self, input_text: String) -> String {
        let messages = TextMessages::new()
            .add_message(
                TextMessageRole::System,
                "You are a helpful AI assistant called Rusty.",
            )
            .add_message(TextMessageRole::User, input_text);
        let response = self.model.send_chat_request(messages).await.unwrap();
        response.choices[0].message.content.clone().unwrap()
    }
}
