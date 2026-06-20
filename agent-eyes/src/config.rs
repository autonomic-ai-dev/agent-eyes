use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub capture: CaptureConfig,
    pub logging: LoggingConfig,
    pub server: ServerConfig,
    pub spine: SpineConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub user_agent: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3105 }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("agent-eyes")
            .join("config.yaml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let s = std::fs::read_to_string(&path)?;
            Ok(serde_yaml::from_str(&s)?)
        } else {
            let cfg = Config::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let s = serde_yaml::to_string(&cfg)?;
            std::fs::write(&path, &s)?;
            Ok(cfg)
        }
    }
}
