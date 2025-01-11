use std::str::FromStr;

use candle_core::quantized::gguf_file;
use candle_core::{Device, Result};
use candle_transformers::models::llama::{Config, LlamaConfig, LlamaEosToks};
use candle_transformers::models::quantized_llama::ModelWeights as Llama32;
use hf_hub::{api::sync::ApiBuilder, api::sync::ApiRepo, Repo, RepoType};
use mistralrs::{GgufModelBuilder, Model, TextMessageRole, TextMessages, TokenSource};
use tokenizers::Tokenizer;

use futures::sink::SinkExt;
use futures::stream::{Stream, StreamExt};
use iced::futures::channel::mpsc;
use iced::stream;

const EOS_TOKEN: &str = "</s>";

pub struct Assistant {
    _model_repo_id: String,
    _model_file: String,
    _tokenizer_repo_id: String,
    model: Model,
}

#[derive(Debug, Clone)]
pub enum Event {
    Loaded(mpsc::Sender<String>),
    AnswerGenerated(String),
}

#[derive(Debug)]
enum State {
    Loading,
    Ready(mpsc::Receiver<String>),
}

pub fn start_assistant(
    model_repo_id: String,
    model_file: String,
    tokenizer_repo_id: String,
) -> impl Stream<Item = Event> {
    stream::channel(100, |mut output| async move {
        let mut state = State::Loading;

        let assistant: Assistant =
            Assistant::new(model_repo_id, model_file, tokenizer_repo_id).await;

        loop {
            match &mut state {
                State::Loading => {
                    let (sender, receiver) = mpsc::channel(1);
                    output.send(Event::Loaded(sender)).await.unwrap();
                    state = State::Ready(receiver);
                }
                State::Ready(receiver) => {
                    futures::select! {
                            input_message = receiver.select_next_some() => {
                                println!("Received input message: {}", input_message);
                                println!("Generating answer");
                                let generated_answer = assistant.generate_answer(input_message).await;
                                println!("Generated answer: {}", generated_answer);
                                output.send(Event::AnswerGenerated(generated_answer)).await.unwrap();
                            }
                    }
                }
            }
        }
    })
}

impl Assistant {
    pub async fn new(model_repo_id: String, model_file: String, tokenizer_repo_id: String) -> Self {
        let model = match GgufModelBuilder::new(model_repo_id.clone(), vec![model_file.clone()])
            .with_tok_model_id(tokenizer_repo_id.clone())
            .with_logging()
            .with_token_source(TokenSource::from_str("env").unwrap())
            .build()
            .await
        {
            Ok(model) => model,
            Err(e) => panic!("Failed loading model {}", e),
        };
        Self {
            model_repo_id,
            model_file,
            tokenizer_repo_id,
            model,
        }
    }

    pub async fn generate_answer(&self, input_text: String) -> String {
        let messages = TextMessages::new()
            .add_message(
                TextMessageRole::System,
                "You are a helpful AI assistant called Rusty.",
            )
            .add_message(TextMessageRole::User, input_text);
        let response = self.model.send_chat_request(messages).await.unwrap();
        response.choices[0].message.content.clone().unwrap()
    }
}

fn get_hf_repo(model_id: String, api_token: &Option<String>) -> Result<ApiRepo> {
    let api = ApiBuilder::new()
        .with_token(api_token.clone())
        .build()
        .map_err(candle_core::Error::wrap)?;

    Ok(api.repo(Repo::with_revision(
        model_id,
        RepoType::Model,
        "main".to_string(),
    )))
}

fn get_tokenizer(repo: &ApiRepo) -> Result<Tokenizer> {
    let tokenizer_filename = repo
        .get("tokenizer.json")
        .map_err(candle_core::Error::wrap)?;

    Ok(Tokenizer::from_file(tokenizer_filename).map_err(candle_core::Error::wrap)?)
}

fn get_model_config(repo: &ApiRepo) -> Result<Config> {
    let config_filename = repo.get("config.json").map_err(candle_core::Error::wrap)?;
    let config: LlamaConfig = serde_json::from_slice(&std::fs::read(config_filename)?)
        .map_err(candle_core::Error::wrap)?;
    let config = config.into_config(false);
    Ok(config)
}

pub fn load_model_and_tokenizer(
    model_repo_id: String,
    tokenizer_repo_id: String,
    token: Option<String>,
) -> Result<(Llama32, Tokenizer, u32)> {
    let device = Device::Cpu;

    let tokenizer_repo = get_hf_repo(tokenizer_repo_id, &token)?;
    let tokenizer = get_tokenizer(&tokenizer_repo)?;

    let config = get_model_config(&tokenizer_repo)?;
    let eos_token = match config
        .eos_token_id
        .or_else(|| tokenizer.token_to_id(EOS_TOKEN).map(LlamaEosToks::Single))
    {
        Some(LlamaEosToks::Single(token_id)) => token_id,
        _ => panic!("cannot find the '{}' eos token", EOS_TOKEN),
    };

    let model_repo = get_hf_repo(model_repo_id, &token)?;
    let model_filename = model_repo
        .get("Llama-3.2-1B-Instruct-Q6_K.gguf")
        .map_err(candle_core::Error::wrap)?;
    let mut model_file = std::fs::File::open(&model_filename)?;
    let model_content =
        gguf_file::Content::read(&mut model_file).map_err(|e| e.with_path(model_filename))?;
    let model = Llama32::from_gguf(model_content, &mut model_file, &device)?;

    Ok((model, tokenizer, eos_token).into())
}
