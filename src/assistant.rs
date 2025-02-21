use anyhow::{anyhow, Result};
use futures::StreamExt;
use std::sync::mpsc;

use crate::ollama::{
    client::{chat, list_models, pull_model, version},
    types::{ChatResponse, Message, PullModelResponse},
};

#[derive(Debug, Default)]
pub struct Messages {
    messages: Vec<Message>,
}

impl Messages {
    fn get_messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

#[derive(Debug)]
pub struct Assistant {
    messages: Messages,
}

impl Assistant {
    pub fn new() -> Self {
        Assistant {
            messages: Messages::default(),
        }
    }

    pub async fn is_ollama_running(&self) -> bool {
        match version().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let response = list_models().await?;
        let models = response
            .models
            .iter()
            .map(move |item| item.name.clone())
            .collect();
        Ok(models)
    }

    pub async fn pull_model(
        &self,
        model: String,
        sender: mpsc::Sender<Option<PullModelResponse>>,
    ) -> Result<()> {
        let mut response_stream = pull_model(model).await?;
        loop {
            match response_stream.next().await {
                Some(pull_response) => match pull_response {
                    Ok(pull_response) => {
                        tracing::debug!("pull model status: {:?}", pull_response);
                        sender.send(Some(pull_response))?;
                    }
                    Err(_) => {
                        tracing::error!("Error while receiving pull model response");
                        drop(sender);
                        return Err(anyhow!("Error while receiving chat response"));
                    }
                },
                None => {
                    tracing::info!("Finished receiving pull model response");
                    sender.send(None)?;
                    return Ok(());
                }
            };
        }
    }

    pub async fn generate_answer(
        &mut self,
        model: String,
        message: Message,
        sender: mpsc::Sender<Option<ChatResponse>>,
    ) -> Result<()> {
        self.messages.add_message(message);
        let messages = self.messages.get_messages();
        let mut response_stream = chat(model, messages).await?;
        loop {
            match response_stream.next().await {
                Some(chat_response) => match chat_response {
                    Ok(chat_response) => {
                        tracing::debug!("chat response: {:?}", chat_response);
                        sender.send(Some(chat_response))?;
                    }
                    Err(_) => {
                        tracing::error!("Error while receiving chat response");
                        drop(sender);
                        return Err(anyhow!("Error while receiving chat response"));
                    }
                },
                None => {
                    tracing::info!("Finished receiving chat response");
                    sender.send(None)?;
                    return Ok(());
                }
            };
        }
    }
}
