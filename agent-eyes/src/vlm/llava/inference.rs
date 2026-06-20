use anyhow::{bail, Context, Result};
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::clip::vision_model::ClipVisionConfig;
use candle_transformers::models::llama::Cache;
use candle_transformers::models::llava::config::{
    HFGenerationConfig, HFLLaVAConfig, HFPreProcessorConfig,
};
use candle_transformers::models::llava::{config::LLaVAConfig, LLaVA};
use hf_hub::api::sync::Api;
use std::path::{Path, PathBuf};
use tokenizers::Tokenizer;

use crate::config::VlmConfig;
use crate::vlm::VlmDescribeResult;

use super::constants::*;
use super::conversation::Conversation;
use super::hub::{device_label, hub_load_local_safetensors, hub_load_safetensors, select_device};
use super::image_processor::{load_image_tensor, ImageProcessor};
use super::token_stream::TokenOutputStream;

pub fn describe(image_path: &Path, prompt: &str, config: &VlmConfig) -> Result<VlmDescribeResult> {
    let device = select_device(config.cpu).map_err(|e| anyhow::anyhow!("{e}"))?;
    let device_name = device_label(&device);

    let (llava_config, tokenizer, clip_vision_config, image_processor) = load_hf_config(config)?;

    let llama_config = llava_config.to_llama_config();
    let dtype = match llava_config.torch_dtype.as_str() {
        "float16" => DType::F16,
        "bfloat16" => DType::BF16,
        _ => bail!("unsupported dtype {}", llava_config.torch_dtype),
    };

    let mut cache =
        Cache::new(true, dtype, &llama_config, &device).map_err(|e| anyhow::anyhow!("{e}"))?;

    let weight_filenames = load_weights(config)?;
    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&weight_filenames, dtype, &device)
            .map_err(|e| anyhow::anyhow!("{e}"))?
    };
    let llava =
        LLaVA::load(vb, &llava_config, clip_vision_config).map_err(|e| anyhow::anyhow!("{e}"))?;

    let prompt_text = build_prompt(prompt, &llava_config);
    let conv_mode = conv_mode_for_model(&config.model_id);
    let mut conv = match conv_mode {
        "chatml_direct" => Conversation::conv_chatml_direct(),
        "llava_v1" => Conversation::conv_llava_v1(),
        other => bail!("unsupported conv mode {other}"),
    };
    conv.append_user_message(Some(&prompt_text));
    conv.append_assistant_message(None);
    let full_prompt = conv.get_prompt();

    let (image_size, image_tensor) =
        load_image_tensor(image_path, &image_processor, &llava_config, dtype)?;
    let image_tensor = image_tensor
        .to_device(&device)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut logits_processor = {
        let temperature = f64::from(config.temperature);
        let sampling = if temperature <= 0. {
            Sampling::ArgMax
        } else {
            Sampling::All { temperature }
        };
        LogitsProcessor::from_sampling(299_792_458, sampling)
    };

    let tokens = tokenizer_image_token(
        &full_prompt,
        &tokenizer,
        llava_config.image_token_index as i64,
        &llava_config,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut input_embeds = llava
        .prepare_inputs_labels_for_multimodal(&tokens, &[image_tensor], &[image_size])
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut tokenizer_stream = TokenOutputStream::new(tokenizer);
    let mut caption = String::new();
    let mut index_pos = 0;
    let eos_token_id = llava_config.eos_token_id;

    for index in 0..config.max_new_tokens {
        let (_, input_embeds_len, _) = input_embeds.dims3().map_err(|e| anyhow::anyhow!("{e}"))?;
        let (context_size, context_index) = if cache.use_kv_cache && index > 0 {
            (1, index_pos)
        } else {
            (input_embeds_len, 0)
        };
        let input = input_embeds
            .i((.., input_embeds_len.saturating_sub(context_size).., ..))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let logits = llava
            .forward(&input, context_index, &mut cache)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let logits = logits.squeeze(0).map_err(|e| anyhow::anyhow!("{e}"))?;
        let (_, input_len, _) = input.dims3().map_err(|e| anyhow::anyhow!("{e}"))?;
        index_pos += input_len;
        let next_token = logits_processor
            .sample(&logits)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let next_token_tensor =
            Tensor::from_vec(vec![next_token], 1, &device).map_err(|e| anyhow::anyhow!("{e}"))?;
        let next_embeds = llava
            .llama
            .embed(&next_token_tensor)
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .unsqueeze(0)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        input_embeds =
            Tensor::cat(&[input_embeds, next_embeds], 1).map_err(|e| anyhow::anyhow!("{e}"))?;
        if next_token == eos_token_id as u32 {
            break;
        }
        if let Some(t) = tokenizer_stream
            .next_token(next_token)
            .map_err(|e| anyhow::anyhow!("{e}"))?
        {
            caption.push_str(&t);
        }
    }
    if let Some(rest) = tokenizer_stream
        .decode_rest()
        .map_err(|e| anyhow::anyhow!("{e}"))?
    {
        caption.push_str(&rest);
    }

    Ok(VlmDescribeResult {
        caption: caption.trim().to_string(),
        model: config.model_id.clone(),
        prompt: prompt.to_string(),
        image: image_path.display().to_string(),
        device: device_name,
    })
}

fn load_hf_config(
    config: &VlmConfig,
) -> Result<(
    LLaVAConfig,
    Tokenizer,
    Option<ClipVisionConfig>,
    ImageProcessor,
)> {
    if let Some(dir) = config.model_dir.as_deref() {
        let path = PathBuf::from(dir);
        let config_filename = path.join("config.json");
        let hf_llava_config: HFLLaVAConfig = serde_json::from_slice(
            &std::fs::read(&config_filename)
                .with_context(|| format!("read {}", config_filename.display()))?,
        )?;
        let generation_config_filename = path.join("generation_config.json");
        let generation_config: HFGenerationConfig =
            serde_json::from_slice(&std::fs::read(&generation_config_filename)?)?;
        let preprocessor_config_filename = path.join("preprocessor_config.json");
        let preprocessor_config: HFPreProcessorConfig =
            serde_json::from_slice(&std::fs::read(&preprocessor_config_filename)?)?;
        let llava_config =
            hf_llava_config.to_llava_config(&generation_config, &preprocessor_config);
        let tokenizer = Tokenizer::from_file(path.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("tokenizer: {e}"))?;
        let clip_vision_config = hf_llava_config.to_clip_vision_config();
        let image_processor = ImageProcessor::from_hf_preprocessor_config(&preprocessor_config);
        return Ok((
            llava_config,
            tokenizer,
            Some(clip_vision_config),
            image_processor,
        ));
    }

    let api = Api::new()?.model(config.model_id.clone());
    let config_filename = api.get("config.json")?;
    let hf_llava_config: HFLLaVAConfig = serde_json::from_slice(&std::fs::read(config_filename)?)?;
    let generation_config_filename = api.get("generation_config.json")?;
    let generation_config: HFGenerationConfig =
        serde_json::from_slice(&std::fs::read(generation_config_filename)?)?;
    let preprocessor_config_filename = api.get("preprocessor_config.json")?;
    let preprocessor_config: HFPreProcessorConfig =
        serde_json::from_slice(&std::fs::read(preprocessor_config_filename)?)?;
    let llava_config = hf_llava_config.to_llava_config(&generation_config, &preprocessor_config);
    let tokenizer_filename = api.get("tokenizer.json")?;
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow::anyhow!("{e}"))?;
    let clip_vision_config = hf_llava_config.to_clip_vision_config();
    let image_processor = ImageProcessor::from_hf_preprocessor_config(&preprocessor_config);
    Ok((
        llava_config,
        tokenizer,
        Some(clip_vision_config),
        image_processor,
    ))
}

fn load_weights(config: &VlmConfig) -> Result<Vec<PathBuf>> {
    if let Some(dir) = config.model_dir.as_deref() {
        return hub_load_local_safetensors(Path::new(dir), "model.safetensors.index.json")
            .map_err(|e| anyhow::anyhow!("{e}"));
    }
    let api = Api::new()?.model(config.model_id.clone());
    hub_load_safetensors(&api, "model.safetensors.index.json").map_err(|e| anyhow::anyhow!("{e}"))
}

fn build_prompt(prompt: &str, llava_config: &LLaVAConfig) -> String {
    let image_token_se =
        format!("{DEFAULT_IM_START_TOKEN}{DEFAULT_IMAGE_TOKEN}{DEFAULT_IM_END_TOKEN}");
    if prompt.contains(IMAGE_PLACEHOLDER) {
        if llava_config.mm_use_im_start_end {
            prompt.replace(IMAGE_PLACEHOLDER, &image_token_se)
        } else {
            prompt.replace(IMAGE_PLACEHOLDER, DEFAULT_IMAGE_TOKEN)
        }
    } else if llava_config.mm_use_im_start_end {
        format!("{image_token_se}\n{prompt}")
    } else {
        format!("{DEFAULT_IMAGE_TOKEN}\n{prompt}")
    }
}

fn conv_mode_for_model(model_id: &str) -> &'static str {
    let model_name = model_id
        .split('/')
        .next_back()
        .unwrap_or(model_id)
        .to_lowercase();
    if model_name.contains("llama-2") {
        "llava_llama_2"
    } else if model_name.contains("mistral") {
        "mistral_instruct"
    } else if model_name.contains("v1.6-34b") {
        "chatml_direct"
    } else if model_name.contains("1.5") || model_name.contains("v1") {
        "llava_v1"
    } else if model_name.contains("mpt") {
        "mpt"
    } else {
        "llava_v1"
    }
}

fn duplicate_vec<T: Clone>(vec: &[T], n: usize) -> Vec<T> {
    let mut res = Vec::new();
    for _ in 0..n {
        res.extend(vec.to_owned());
    }
    res
}

fn insert_separator<T: Clone>(x: Vec<Vec<T>>, sep: Vec<T>) -> Vec<Vec<T>> {
    let sep = vec![sep];
    let sep = duplicate_vec(&sep, x.len());
    let mut res = x
        .iter()
        .zip(sep.iter())
        .flat_map(|(x, y)| vec![x.clone(), y.clone()])
        .collect::<Vec<Vec<T>>>();
    res.pop();
    res
}

fn tokenizer_image_token(
    prompt: &str,
    tokenizer: &Tokenizer,
    image_token_index: i64,
    llava_config: &LLaVAConfig,
) -> candle_core::Result<Tensor> {
    let prompt_chunks = prompt
        .split(' ')
        .map(|s| {
            tokenizer
                .encode(s, true)
                .unwrap()
                .get_ids()
                .to_vec()
                .iter()
                .map(|x| *x as i64)
                .collect::<Vec<i64>>()
        })
        .collect::<Vec<Vec<i64>>>();
    let mut input_ids = Vec::new();
    let mut offset = 0;
    if !prompt_chunks.is_empty()
        && !prompt_chunks[0].is_empty()
        && prompt_chunks[0][0] == llava_config.bos_token_id as i64
    {
        offset = 1;
        input_ids.push(prompt_chunks[0][0]);
    }

    for x in insert_separator(
        prompt_chunks,
        duplicate_vec(&[image_token_index], offset + 1),
    )
    .iter()
    {
        input_ids.extend(x[1..].to_vec())
    }
    let input_len = input_ids.len();
    Tensor::from_vec(input_ids, (1, input_len), &Device::Cpu)
}
