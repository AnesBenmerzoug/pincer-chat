use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    pub total_duration: Option<u32>,
    pub load_duration: Option<u32>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u32>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u32>,
}
