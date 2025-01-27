pub mod types;

use std::str::FromStr;
use std::time::Duration;

use anyhow::{Error, Result};
use futures::stream::StreamExt;
use futures::{Stream, TryStreamExt};
use reqwest;
use serde_json;

use crate::assistant::types::{
    ChatRequest, ChatResponse, Message, PullModelRequest, PullModelResponse,
};
/*
#[derive(Debug)]
pub struct Assistant {
    model: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub enum Event {
    Loaded(mpsc::Sender<Vec<Message>>),
    ErrorLoading(String),
    GeneratedAnswerDelta(Message),
    FinishedGeneration,
}

#[derive(Debug)]
enum State {
    Loading,
    Ready(mpsc::Receiver<Vec<Message>>),
}

impl Assistant {
    pub async fn new(model: String) -> Self {
        let client = reqwest::Client::new();
        Self { model, client }
    }

    pub async fn pull_model(&self) -> Result<impl Stream<Item = Result<PullModelResponse>>> {
        let body = PullModelRequest {
            model: self.model.clone(),
            insecure: false,
            stream: true,
        };
        let serialized_body = serde_json::to_string(&body)?;
        let response = self
            .client
            .post("http://localhost:11434/api/pull")
            .timeout(Duration::from_secs(60))
            .body(serialized_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::msg(response.text().await?));
        }

        let stream = response.bytes_stream().map(|response| match response {
            Ok(bytes) => {
                let result = serde_json::from_slice::<PullModelResponse>(&bytes);
                match result {
                    Ok(result) => Ok(result),
                    Err(_) => Err(Error::msg("Failed parsing response")),
                }
            }
            Err(e) => Err(e.into()),
        });
        Ok(stream)
    }

    pub async fn generate_answer(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<ChatResponse>>> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: messages,
            stream: true,
        };
        let serialized_body = serde_json::to_string(&body)?;
        let response = self
            .client
            .post("http://localhost:11434/api/chat")
            .timeout(Duration::from_secs(60))
            .body(serialized_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::msg(response.text().await?));
        }

        let stream = response.bytes_stream().map(|response| match response {
            Ok(bytes) => {
                let result = serde_json::from_slice::<ChatResponse>(&bytes);
                match result {
                    Ok(result) => Ok(result),
                    Err(_) => Err(Error::msg("Failed parsing response")),
                }
            }
            Err(e) => Err(e.into()),
        });
        Ok(stream)
    }
}

pub fn start_assistant(model: String) -> impl StreamExt<Item = Event> {
    channel(100, |mut output| async move {
        let mut state = State::Loading;

        // Create the runtime
        let rt = Runtime::new().unwrap();

        // Spawn the root task
        rt.block_on(async {

            println!("Creating assistant");
            let assistant: Assistant = Assistant::new(model).await;

            println!("Pulling model");
            /*
            let response = match assistant.pull_model().await {
                Ok(mut response) => {
                    match response.try_next().await.unwrap() {
                        Some(res) => res,
                        None => {
                            output.send(Event::ErrorLoading(
                                String::from_str("Failed pulling model").unwrap(),
                            )).await.unwrap();
                            return;
                        }
                    }
                }
                Err(e) => {
                    output.send(Event::ErrorLoading(e.to_string())).await.unwrap();
                    return;
                }
            };
            println!("Pulled model with status: {}", response.status);
            */

            loop {
                match &mut state {
                    State::Loading => {
                        println!("Assistant loading");
                        let (sender, receiver) = mpsc::channel(1);
                        output.send(Event::Loaded(sender)).await.unwrap();
                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        futures::select! {
                                messages = receiver.select_next_some() => {
                                    dbg!("Received input messages: {}", &messages);
                                    println!("Generating answer");
                                    match assistant.generate_answer(messages).await {
                                        Ok(mut response) => {
                                            match response.try_next().await.unwrap() {
                                                Some(delta) => {
                                                    if !delta.done {
                                                        output.send(Event::GeneratedAnswerDelta(delta.message.unwrap())).await.unwrap();
                                                    } else {
                                                        output.send(Event::FinishedGeneration).await.unwrap();
                                                    }
                                                },
                                                None => {
                                                    output.send(Event::ErrorLoading(String::from_str("Failed pulling model").unwrap())).await.unwrap();
                                                    return;
                                                }
                                            };
                                        }
                                        Err(e) => {
                                            output.send(Event::ErrorLoading(e.to_string())).await.unwrap();
                                            return;
                                        }
                                    };
                                }
                        }
                    }
                }
            }
        })
    })
}
 */