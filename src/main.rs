pub mod model;
pub mod text_generation;

use crate::model::load_model_and_tokenizer;
use crate::text_generation::TextGeneration;
use candle_core::{Device, Result};

fn main() -> Result<()> {
    let seed = 16;
    let sample_len = 32;

    let hf_token: Option<String> = match std::env::var("HF_TOKEN") {
        Ok(val) => Some(val),
        _ => None,
    };
    let tokenizer_repo_id = "meta-llama/Llama-3.2-1B".to_string();
    let model_repo_id = "bartowski/Llama-3.2-1B-Instruct-GGUF".to_string();

    let (model, tokenizer, eos_token) =
        load_model_and_tokenizer(model_repo_id, tokenizer_repo_id, hf_token)?;

    let text_generator = TextGeneration::new(
        model,
        tokenizer,
        eos_token,
        seed,
        None,
        None,
        None,
        0.1,
        0,
        &Device::Cpu,
    );

    let prompt = "Hi, how are you?".to_string();
    println!("Prompt: {}", prompt);
    let generated_text = text_generator.generate(prompt, sample_len).unwrap();
    println!("Generated text: {}", generated_text);

    Ok(())
}
