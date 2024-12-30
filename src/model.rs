use candle_core::quantized::gguf_file;
use candle_core::{Device, Result};
use candle_transformers::models::llama::{Config, LlamaConfig, LlamaEosToks};
use candle_transformers::models::quantized_llama::ModelWeights as Llama32;
use hf_hub::{api::sync::ApiBuilder, api::sync::ApiRepo, Repo, RepoType};
use tokenizers::Tokenizer;

const EOS_TOKEN: &str = "</s>";

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
