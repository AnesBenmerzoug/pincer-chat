use serde::{Deserialize, Serialize};

use crate::assistant::database::models::Message as DatabaseMessage;

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

impl TryFrom<String> for Role {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "system" => Ok(Role::System),
            "tool" => Ok(Role::Tool),
            _ => Err(format!("Could not convert string {value} to Role enum")),
        }
    }
}

impl Into<&str> for Role {
    fn into(self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::Tool => "tool",
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl From<DatabaseMessage> for Message {
    fn from(value: DatabaseMessage) -> Self {
        Self {
            content: value.content,
            role: Role::try_from(value.role).expect("Message role to be valid"),
        }
    }
}

// Ollama Structs
#[derive(Debug, Serialize)]
pub struct PullModelRequest {
    pub model: String,
    pub insecure: bool,
    pub stream: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PullModelResponse {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ListModelResponse {
    pub models: Vec<ListModelSingleModelResponse>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ListModelSingleModelResponse {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: ModelDetails,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct VersionResponse {
    pub version: String,
}
