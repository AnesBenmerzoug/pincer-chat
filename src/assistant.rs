pub mod database;
pub mod ollama;

use anyhow::Result;
use database::Database;
use futures::Stream;
use futures::StreamExt;

use self::database::models::Message as DatabaseMessage;
use self::ollama::{
    api::{chat, list_models, pull_model, version},
    types::{Message as OllamaMessage, PullModelResponse, Role},
};

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
    database: Database,
    parameters: AssistantParameters,
}

impl Assistant {
    pub async fn new() -> Self {
        let database = match Database::new().await {
            Ok(database) => database,
            Err(error) => {
                panic!("Failed connecting to database because of: {error}")
            }
        };
        Assistant {
            database,
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

    pub async fn generate_answer(
        &mut self,
        messages: Vec<OllamaMessage>,
    ) -> Result<impl Stream<Item = Result<OllamaMessage>>> {
        let response_stream = chat(self.parameters.model.clone().unwrap(), messages).await?;
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
