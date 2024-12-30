use candle_core::quantized::gguf_file;
use candle_core::{Device, Result};
use candle_transformers::models::quantized_llama::ModelWeights as Llama32;
use hf_hub::{api::sync::ApiBuilder, api::sync::ApiRepo, Repo, RepoType};
use tokenizers::Tokenizer;

#[derive(Clone)]
pub struct Model {
    pub llm: Llama32,
    pub tokenizer: Tokenizer,
}

impl From<(Llama32, Tokenizer)> for Model {
    fn from(e: (Llama32, Tokenizer)) -> Self {
        Self {
            llm: e.0,
            tokenizer: e.1,
        }
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

pub fn initialise_model(
    model_repo_id: String,
    tokenizer_repo_id: String,
    token: Option<String>,
) -> Result<Model> {
    let tokenizer_repo = get_hf_repo(tokenizer_repo_id, &token)?;
    let tokenizer = get_tokenizer(&tokenizer_repo)?;
    let device = Device::Cpu;

    let model_repo = get_hf_repo(model_repo_id, &token)?;
    let model_filename = model_repo
        .get("Llama-3.2-1B-Instruct-Q6_K.gguf")
        .map_err(candle_core::Error::wrap)?;
    let mut model_file = std::fs::File::open(&model_filename)?;
    let model_content =
        gguf_file::Content::read(&mut model_file).map_err(|e| e.with_path(model_filename))?;
    let model = Llama32::from_gguf(model_content, &mut model_file, &device)?;

    Ok((model, tokenizer).into())
}
