use std::time::Duration;

use anyhow::{Error, Result};
use futures::stream::StreamExt;
use futures::{Stream, TryStreamExt};
use reqwest;
use serde_json;

use crate::ollama::types::{
    ChatRequest, ChatResponse, Message, PullModelRequest, PullModelResponse,
};

#[derive(Debug)]
pub struct OllamaClient {
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    pub async fn pull_model(
        &self,
        model: String,
    ) -> Result<impl Stream<Item = Result<PullModelResponse>>> {
        let body = PullModelRequest {
            model: model.clone(),
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

    pub async fn chat(
        &self,
        model: String,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<ChatResponse>>> {
        let body = ChatRequest {
            model: model.clone(),
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
