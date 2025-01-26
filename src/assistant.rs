use std::collections::HashMap;
use std::str::FromStr;

use reqwest;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug)]
pub struct Assistant {
    model_name: String,
    client: reqwest::blocking::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub role: Role,
    pub content: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AssistantMessages {
    pub messages: Vec<AssistantMessage>
}

impl AssistantMessages {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
    pub fn add_message(&mut self, message: AssistantMessage) -> &mut Self {
        self.messages.push(message);
        self
    }

    pub fn add_system_message(&mut self, content: String) -> &mut Self {
        let message = AssistantMessage{role: Role::System, content};
        self.add_message(message)
    }

    pub fn add_user_message(&mut self, content: String) -> &mut Self {
        let message = AssistantMessage{role: Role::User, content};
        self.add_message(message)
    }

    pub fn add_assistant_message(&mut self, content: String) -> &mut Self {
        let message = AssistantMessage{role: Role::Assistant, content};
        self.add_message(message)
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Loaded,
    GeneratedAnswerDelta(String),
    FinishedGeneration,
}

impl Assistant {
    pub fn new(model_name: String) -> Self {
        let client = reqwest::blocking::Client::new();
        let mut body = HashMap::new();
        body.insert("model", model_name.clone()); 
        let response = client.post("http://localhost:11434/api/pull").json(&body).send().unwrap();

        #[derive(Deserialize)]
        struct PullResponse {
            status: String,
        }
        let response_json: PullResponse = response.json().unwrap();
        if response_json.status == "success" {
            println!("Succeeded downloading model {}", model_name);
        } else {
            println!("Failed downloading model {}", model_name)
        }
        Self {
            model_name,
            client,
        }
    }

    pub fn generate_answer(&self, messages: &AssistantMessages) -> AssistantMessage {
        let mut body = HashMap::new();
        body.insert("model", self.model_name.clone()); 
        body.insert("messages", serde_json::to_string(&messages).unwrap());
        body.insert("stream", String::from_str("false").unwrap());
        let response = self.client.post("http://localhost:11434/api/chat").json(&body).send().unwrap();

        #[derive(Deserialize)]
        struct ChatResponse {
            model: String,
            created_at: String,
            message: AssistantMessage,
            done: bool,
            total_duration: u32,
            load_duration: u32,
            prompt_eval_count: u32,
            prompt_eval_duration: u32,
            eval_count: u32,
            eval_duration: u32
        }
        let response_json: ChatResponse = response.json().unwrap();
        response_json.message
    }
}
