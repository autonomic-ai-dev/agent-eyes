//! Describes images through a local Ollama server (`POST /api/generate`).

use std::path::Path;

use anyhow::{bail, Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::config::VlmConfig;

use super::VlmDescribeResult;

#[derive(Debug, Serialize, PartialEq)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    images: Vec<String>,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Serialize, PartialEq)]
struct GenerateOptions {
    temperature: f32,
    num_predict: usize,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    response: String,
}

fn request_body<'a>(config: &'a VlmConfig, prompt: &'a str, image: &[u8]) -> GenerateRequest<'a> {
    GenerateRequest {
        model: &config.ollama_model,
        prompt,
        images: vec![base64::engine::general_purpose::STANDARD.encode(image)],
        stream: false,
        options: GenerateOptions {
            temperature: config.temperature,
            num_predict: config.max_new_tokens,
        },
    }
}

pub async fn describe(
    image_path: &Path,
    prompt: &str,
    config: &VlmConfig,
) -> Result<VlmDescribeResult> {
    let image = tokio::fs::read(image_path)
        .await
        .with_context(|| format!("read {}", image_path.display()))?;
    let url = format!("{}/api/generate", config.ollama_url.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(url)
        .json(&request_body(config, prompt, &image))
        .send()
        .await
        .context("ollama unreachable; start it with `ollama serve`")?;
    if !response.status().is_success() {
        bail!("ollama returned HTTP {}", response.status());
    }
    let parsed: GenerateResponse = response
        .json()
        .await
        .context("ollama returned an unexpected response")?;
    Ok(VlmDescribeResult {
        caption: parsed.response.trim().to_string(),
        model: config.ollama_model.clone(),
        prompt: prompt.to_string(),
        image: image_path.display().to_string(),
        device: "ollama".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn request_asks_for_one_non_streamed_answer_with_base64_image() {
        let config = VlmConfig::default();
        let body = request_body(&config, "Describe.", b"png-bytes");
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["model"], "qwen2.5vl:7b");
        assert_eq!(json["stream"], false);
        assert_eq!(json["images"][0], "cG5nLWJ5dGVz");
        assert_eq!(json["options"]["num_predict"], 256);
    }

    #[tokio::test]
    async fn describe_returns_the_model_caption_from_the_server() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 65536];
            let n = socket.read(&mut buf).await.unwrap();
            let reply = r#"{"model":"llava","response":"  A web page.  ","done":true}"#;
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                reply.len()
            );
            socket.write_all(head.as_bytes()).await.unwrap();
            socket.write_all(reply.as_bytes()).await.unwrap();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });

        let image = std::env::temp_dir().join(format!("ollama-test-{}.png", std::process::id()));
        std::fs::write(&image, b"png-bytes").unwrap();
        let config = VlmConfig {
            ollama_url: format!("http://{addr}"),
            ..VlmConfig::default()
        };
        let result = describe(&image, "Describe.", &config).await.unwrap();
        let request = server.await.unwrap();

        assert!(request.starts_with("POST /api/generate"));
        assert_eq!(result.caption, "A web page.");
        assert_eq!(result.device, "ollama");
    }

    #[tokio::test]
    async fn unreachable_server_is_a_clear_error() {
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            listener.local_addr().unwrap().port()
        };
        let image =
            std::env::temp_dir().join(format!("ollama-test-down-{}.png", std::process::id()));
        std::fs::write(&image, b"png-bytes").unwrap();
        let config = VlmConfig {
            ollama_url: format!("http://127.0.0.1:{port}"),
            ..VlmConfig::default()
        };
        let err = describe(&image, "Describe.", &config).await.unwrap_err();
        assert!(err.to_string().contains("ollama serve"));
    }
}
