pub mod token_output_stream;

use crate::text_generation::token_output_stream::TokenOutputStream;
use candle_core::{DType, Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights as Llama32;
use indicatif::ProgressIterator;
use tokenizers::Tokenizer;

pub struct TextGeneration {
    model: Llama32,
    device: Device,
    token_output_stream: TokenOutputStream,
    eos_token: u32,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model: Llama32,
        tokenizer: Tokenizer,
        eos_token: u32,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        _top_k: Option<usize>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
    ) -> Self {
        let logits_processor = LogitsProcessor::new(seed, temp, top_p);

        Self {
            model,
            token_output_stream: TokenOutputStream::new(tokenizer),
            eos_token,
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
        }
    }

    pub fn generate(
        mut self,
        prompt: String,
        sample_len: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.token_output_stream.clear();
        let mut tokens = self
            .token_output_stream
            .tokenizer()
            .encode(prompt, true)
            .unwrap()
            .get_ids()
            .to_vec();

        let mut string = String::new();

        for index in (0..sample_len).progress() {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)
                .unwrap()
                .unsqueeze(0)
                .unwrap();
            let logits = self.model.forward(&input, start_pos).unwrap();
            let logits = logits
                .squeeze(0)
                .unwrap()
                .squeeze(0)
                .unwrap()
                .to_dtype(DType::F32)
                .unwrap();
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )
                .unwrap()
            };

            let next_token = self.logits_processor.sample(&logits).unwrap();
            tokens.push(next_token);

            if next_token == self.eos_token {
                break;
            }
            if let Some(t) = self.token_output_stream.next_token(next_token).unwrap() {
                string.push_str(&t);
            }
        }

        Ok(string)
    }
}
