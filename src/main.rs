use candle_core::utils;
use candle_core::Result;
use candle_transformers::models::llama as model;
use hf_hub::{api::sync::ApiBuilder, api::sync::ApiRepo, Repo, RepoType};
use model::LlamaConfig;
use tokenizers::Tokenizer;

const EOS_TOKEN: &str = "</s>";

fn main() {
    println!(
        "avx: {}, neon: {}, simd128: {}, f16c: {}",
        utils::with_avx(),
        utils::with_neon(),
        utils::with_simd128(),
        utils::with_f16c()
    );

    let api_token: Option<String> = match std::env::var("HF_TOKEN") {
        Ok(val) => Some(val),
        _ => None,
    };
    let model_id = "meta-llama/Llama-3.2-1B".to_string();
    //let device = candle_core::Device::Cpu;

    let (tokenizer_filename, config) = {
        println!("loading the model weights from {model_id}");
        let api = get_repo(model_id, api_token).unwrap();

        let tokenizer_filename = api.get("tokenizer.json").unwrap();
        let config_filename = api.get("config.json").unwrap();
        let config: LlamaConfig =
            serde_json::from_slice(&std::fs::read(config_filename).unwrap()).unwrap();

        // let filenames = hub_load_safetensors(&api, "model.safetensors.index.json")?;
        // let cache = model::Cache::new(true, dtype, &config, &device)?;
        // let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
        // (Llama::load(vb, &config)?, tokenizer_filename, cache, config)
        (tokenizer_filename, config)
    };
    let tokenizer = Tokenizer::from_file(tokenizer_filename).unwrap();
    let eos_token_id = config.eos_token_id.or_else(|| {
        tokenizer
            .token_to_id(EOS_TOKEN)
            .map(model::LlamaEosToks::Single)
    });

    let prompt = "Hi, how are you?".to_string();
    print!("{prompt}");
    let encoding = tokenizer.encode(prompt, true).unwrap();
    let token_ids = encoding.get_ids();
    print!("{:?}", token_ids);
}

fn get_repo(model_id: String, api_token: Option<String>) -> Result<ApiRepo> {
    let api = ApiBuilder::new().with_token(api_token).build().unwrap();

    Ok(api.repo(Repo::with_revision(
        model_id,
        RepoType::Model,
        "main".to_string(),
    )))
}

/// Loads the safetensors files for a model from the hub based on a json index file.
pub fn hub_load_safetensors(repo: &ApiRepo, json_file: &str) -> Result<Vec<std::path::PathBuf>> {
    let json_file = repo.get(json_file).map_err(candle_core::Error::wrap)?;
    let json_file = std::fs::File::open(json_file)?;
    let json: serde_json::Value =
        serde_json::from_reader(&json_file).map_err(candle_core::Error::wrap)?;
    let weight_map = match json.get("weight_map") {
        None => candle_core::bail!("no weight map in {json_file:?}"),
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => candle_core::bail!("weight map in {json_file:?} is not a map"),
    };
    let mut safetensors_files = std::collections::HashSet::new();
    for value in weight_map.values() {
        if let Some(file) = value.as_str() {
            safetensors_files.insert(file.to_string());
        }
    }
    let safetensors_files = safetensors_files
        .iter()
        .map(|v| repo.get(v).map_err(candle_core::Error::wrap))
        .collect::<Result<Vec<_>>>()?;
    Ok(safetensors_files)
}
