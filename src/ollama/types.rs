use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    #[serde(alias = "system")]
    System,
    #[serde(alias = "user")]
    User,
    #[default]
    #[serde(alias = "assistant")]
    Assistant,
    #[serde(alias = "tool")]
    Tool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn update(&mut self, other: &Self) -> Result<()> {
        if self.role != other.role {
            return Err(anyhow!("the two message roles should be the same"));
        }
        self.content.push_str(&*other.content);
        Ok(())
    }
}

// Ollama Structs
#[derive(Debug, Serialize)]
pub struct PullModelRequest {
    pub model: String,
    pub insecure: bool,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct PullModelResponse {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: Message,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u64>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u64>,
    pub eval_duration: Option<u64>,
}
