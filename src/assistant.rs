pub mod database;
pub mod notification;
pub mod ollama;
pub mod prompts;

use anyhow::Result;
use futures::Stream;
use futures::StreamExt;

use ollama::{
    api::{chat, list_models, pull_model, version},
    types::{Message as OllamaMessage, PullModelResponse, Role},
};
use prompts::THREAD_TITLE_PROMPT;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AssistantParameters {
    pub model: Option<String>,
    pub temperature: f64,
    pub top_k: u64,
    pub top_p: f64,
    pub seed: u64,
}

impl Default for AssistantParameters {
    fn default() -> Self {
        Self {
            model: None,
            temperature: 0.5,
            top_k: 40,
            top_p: 0.9,
            seed: 42,
        }
    }
}

#[derive(Debug)]
pub struct Assistant {
    parameters: AssistantParameters,
}

impl Assistant {
    pub async fn new() -> Self {
        Assistant {
            parameters: AssistantParameters::default(),
        }
    }

    pub fn set_model(&mut self, model: String) {
        self.parameters.model = Some(model);
    }

    pub fn set_temperature(&mut self, value: f64) {
        self.parameters.temperature = value;
    }

    pub fn set_top_k(&mut self, value: u64) {
        self.parameters.top_k = value;
    }

    pub fn set_top_p(&mut self, value: f64) {
        self.parameters.top_p = value;
    }

    pub fn reset_parameters(&mut self) {
        self.parameters = AssistantParameters {
            model: self.parameters.model.clone(),
            ..AssistantParameters::default()
        }
    }

    pub async fn is_ollama_running(&self) -> bool {
        version().await.is_ok()
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
    ) -> Result<impl Stream<Item = Result<PullModelResponse>>> {
        let response_stream = pull_model(model).await?;
        let pull_model_stream = response_stream.map(|response| match response {
            Ok(response) => {
                tracing::debug!("pull model response: {:?}", response);
                Ok(response)
            }
            Err(error) => {
                tracing::error!("Error while receiving pull model response because of: {error}");
                Err(error)
            }
        });
        Ok(pull_model_stream)
    }

    fn remove_think_tags(&self, text: String) -> String {
        let start_index = text.find("<think>").unwrap_or(0);
        let end_index = text.find("</think>").map_or(0, |v| v + "</think>".len());
        let before_part = &text[..start_index];
        let after_part = &text[end_index..];
        format!("{}{}", before_part, after_part)
    }

    pub async fn generate_thread_title(&mut self, message: OllamaMessage) -> Result<String> {
        let system_message = OllamaMessage {
            content: String::from(THREAD_TITLE_PROMPT),
            role: Role::System,
        };
        let query_message = OllamaMessage {
            content: format!("<query>{}</query>", message.content),
            role: Role::User,
        };
        let messages = vec![system_message, query_message];
        let mut message_stream = self.generate_answer(messages).await?;
        let mut thread_title = String::new();
        while let Some(result) = message_stream.next().await {
            let message = result?;
            thread_title += &message.content;
        }
        // Remove <think></think> tags, if there are any
        thread_title = self.remove_think_tags(thread_title);
        thread_title = thread_title.replace("\n", "");
        Ok(thread_title)
    }

    pub async fn generate_answer(
        &mut self,
        messages: Vec<OllamaMessage>,
    ) -> Result<impl Stream<Item = Result<OllamaMessage>>> {
        let response_stream = chat(self.parameters.model.clone().unwrap(), messages, true).await?;
        let generation_stream = response_stream.map(|chat_response| match chat_response {
            Ok(chat_response) => {
                tracing::debug!("chat response: {:?}", chat_response);
                Ok(chat_response.message)
            }
            Err(error) => {
                tracing::error!("Error while receiving chat response because of: {error}");
                Err(error)
            }
        });
        Ok(generation_stream)
    }
}
