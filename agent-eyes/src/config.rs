use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub capture: CaptureConfig,
    pub logging: LoggingConfig,
    pub server: ServerConfig,
    pub spine: SpineConfig,
    #[serde(default)]
    pub vlm: VlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlmConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_vlm_model")]
    pub model_id: String,
    #[serde(default)]
    pub model_dir: Option<String>,
    #[serde(default = "default_vlm_max_tokens")]
    pub max_new_tokens: usize,
    #[serde(default = "default_vlm_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub cpu: bool,
}

fn default_vlm_model() -> String {
    "llava-hf/llava-1.5-7b-hf".into()
}

fn default_vlm_max_tokens() -> usize {
    256
}

fn default_vlm_temperature() -> f32 {
    0.2
}

impl Default for VlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model_id: default_vlm_model(),
            model_dir: None,
            max_new_tokens: default_vlm_max_tokens(),
            temperature: default_vlm_temperature(),
            cpu: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3105 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineConfig {
    pub url: String,
}

impl Default for SpineConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:3100".into(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capture: CaptureConfig {
                user_agent: "agent-eyes/0.2.0".into(),
            },
            logging: LoggingConfig {
                level: "info".into(),
            },
            server: ServerConfig::default(),
            spine: SpineConfig::default(),
            vlm: VlmConfig::default(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        agent_body_core::config_path()
    }

    pub fn load() -> Result<Self> {
        agent_body_core::organ_config::load("eyes")
    }
}
