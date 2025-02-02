use std::time::Duration;

use anyhow::{Error, Result};
use futures::stream::StreamExt;
use futures::Stream;
use reqwest;
use serde_json;

use crate::ollama::types::{
    ChatRequest, ChatResponse, Message, PullModelRequest, PullModelResponse,
};

pub async fn pull_model(model: String) -> Result<impl Stream<Item = Result<PullModelResponse>>> {
    let body = PullModelRequest {
        model: model.clone(),
        insecure: false,
        stream: true,
    };
    let serialized_body = serde_json::to_string(&body)?;
    let client = reqwest::Client::new();
    let response = client
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
    model: String,
    messages: Vec<Message>,
) -> Result<impl Stream<Item = Result<ChatResponse>>> {
    let body = ChatRequest {
        model: model.clone(),
        messages: messages,
        stream: true,
    };
    let serialized_body = serde_json::to_string(&body)?;

    let client = reqwest::Client::new();
    let response = client
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
                Err(e) => Err(Error::msg(format!("Failed parsing response {e}"))),
            }
        }
        Err(e) => Err(e.into()),
    });
    Ok(stream)
}
