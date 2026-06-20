//! Native local vision (LLaVA via candle). Build with `--features vlm` for inference.

#[cfg(feature = "vlm")]
mod llava;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::config::VlmConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlmDescribeResult {
    pub caption: String,
    pub model: String,
    pub prompt: String,
    pub image: String,
    pub device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlmStatus {
    pub enabled: bool,
    pub feature_compiled: bool,
    pub model_id: String,
    pub model_dir: Option<String>,
    pub default_prompt: String,
}

pub fn default_prompt() -> &'static str {
    "Describe this image in detail."
}

pub fn vlm_status(config: &VlmConfig) -> VlmStatus {
    VlmStatus {
        enabled: config.enabled,
        feature_compiled: cfg!(feature = "vlm"),
        model_id: config.model_id.clone(),
        model_dir: config.model_dir.clone(),
        default_prompt: default_prompt().into(),
    }
}

pub async fn describe_image(
    image_path: &Path,
    prompt: Option<&str>,
    config: &VlmConfig,
) -> Result<VlmDescribeResult> {
    if !config.enabled {
        bail!("vlm.enabled is false in config (~/.autonomic/config.toml)");
    }

    let prompt = prompt.unwrap_or(default_prompt());

    #[cfg(feature = "vlm")]
    {
        let config = config.clone();
        let prompt = prompt.to_string();
        let image_path = image_path.to_path_buf();
        return tokio::task::spawn_blocking(move || {
            crate::vlm::llava::describe(&image_path, &prompt, &config)
        })
        .await?;
    }

    #[cfg(not(feature = "vlm"))]
    {
        let _ = (image_path, prompt);
        bail!(
            "native VLM not compiled; rebuild with `cargo build --features vlm` \
             and download weights for `{}`",
            config.model_id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_feature_flag() {
        let status = vlm_status(&VlmConfig::default());
        assert_eq!(status.feature_compiled, cfg!(feature = "vlm"));
        assert!(status.model_id.contains("llava"));
    }
}
