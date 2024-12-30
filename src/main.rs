pub mod model;

use crate::model::{initialise_model, Model};
use candle_core::utils;
use candle_core::Result;

const EOS_TOKEN: &str = "</s>";

fn main() -> Result<()> {
    println!(
        "avx: {}, neon: {}, simd128: {}, f16c: {}",
        utils::with_avx(),
        utils::with_neon(),
        utils::with_simd128(),
        utils::with_f16c()
    );

    let hf_token: Option<String> = match std::env::var("HF_TOKEN") {
        Ok(val) => Some(val),
        _ => None,
    };
    let tokenizer_repo_id = "meta-llama/Llama-3.2-1B".to_string();
    let model_repo_id = "bartowski/Llama-3.2-1B-Instruct-GGUF".to_string();
    let sample_len = 32;

    let model: Model = initialise_model(model_repo_id, tokenizer_repo_id, hf_token)?;
    // let eos_token = tokenizer.get_vocab(true).get(EOS_TOKEN).unwrap();

    let prompt = "Hi, how are you?".to_string();
    print!("{prompt}\n");
    let encoding = model
        .tokenizer
        .encode(prompt, true)
        .map_err(candle_core::Error::wrap)?;
    print!("{:?}\n", encoding.get_tokens());
    let token_ids = encoding.get_ids();
    print!("{:?}\n", token_ids);
    Ok(())
}
