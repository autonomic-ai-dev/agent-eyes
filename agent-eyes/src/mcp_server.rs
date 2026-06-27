#[allow(unused_imports)]
use rmcp::model::{CallToolResult, Content, ErrorData as McpError, ServerInfo};
use rmcp::serve_server;
use rmcp::tool;
use rmcp::ServerHandler;
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

use crate::config::Config;

#[derive(Clone)]
pub struct EyesMcp {
    config: Config,
}

impl EyesMcp {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(config: Config) -> anyhow::Result<()> {
        let server = Self::new(config);
        let service = serve_server(server, rmcp::transport::io::stdio()).await?;
        service.waiting().await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DescribeDomParams {
    html: String,
    #[serde(default = "default_max_elements")]
    max_elements: usize,
}

fn default_max_elements() -> usize {
    5000
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DiffScreenshotsParams {
    url: String,
    baseline: Option<String>,
    #[serde(default = "default_diff_threshold")]
    threshold: f64,
}

fn default_diff_threshold() -> f64 {
    1.0
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VlmCaptionParams {
    image: String,
    #[serde(default)]
    prompt: Option<String>,
}

#[tool(tool_box)]
impl EyesMcp {
    #[tool(
        description = "Parse raw HTML into a token-efficient JSON layout of interactive elements"
    )]
    async fn eyes_describe_dom(
        &self,
        #[tool(aggr)] params: DescribeDomParams,
    ) -> Result<CallToolResult, McpError> {
        let text = serde_json::to_string_pretty(&crate::dom_index::parse_dom_elements(
            &params.html,
            params.max_elements,
        ))
        .unwrap_or_else(|_| "[]".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Capture a localhost URL screenshot and pixel-diff against a baseline")]
    async fn eyes_diff_screenshots(
        &self,
        #[tool(aggr)] params: DiffScreenshotsParams,
    ) -> Result<CallToolResult, McpError> {
        let baseline_path = params
            .baseline
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::verify::baseline_for_target(&params.url));

        let capture_path = crate::verify::baseline_dir().join("mcp_capture.png");

        if let Err(e) = crate::capture::capture_url(&params.url, &capture_path).await {
            return Err(McpError::internal_error(
                format!("capture failed: {e}"),
                None,
            ));
        }

        if !baseline_path.exists() {
            if let Err(e) = std::fs::copy(&capture_path, &baseline_path) {
                return Err(McpError::internal_error(
                    format!("seed baseline failed: {e}"),
                    None,
                ));
            }
            let result = serde_json::json!({
                "status": "baseline_seeded",
                "baseline": baseline_path.display().to_string(),
                "diff_percent": 0.0,
                "diff_pixels": 0,
                "total_pixels": 0,
            });
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]));
        }

        let diff_path = crate::verify::baseline_dir().join("mcp_diff.png");
        match crate::diff::pixel_diff_metrics(&baseline_path, &capture_path, &diff_path) {
            Ok(metrics) => {
                let result = serde_json::json!({
                    "status": "compared",
                    "diff_percent": metrics.diff_percent,
                    "diff_pixels": metrics.diff_pixels,
                    "total_pixels": metrics.total_pixels,
                    "passed": metrics.diff_percent <= params.threshold,
                    "baseline": baseline_path.display().to_string(),
                    "capture": capture_path.display().to_string(),
                    "diff_output": diff_path.display().to_string(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
            Err(e) => Err(McpError::internal_error(format!("diff failed: {e}"), None)),
        }
    }

    #[tool(description = "Run a local Candle LLaVA model to caption an image")]
    async fn eyes_vlm_caption(
        &self,
        #[tool(aggr)] params: VlmCaptionParams,
    ) -> Result<CallToolResult, McpError> {
        let image_path = PathBuf::from(&params.image);
        match crate::vlm::describe_image(&image_path, params.prompt.as_deref(), &self.config.vlm)
            .await
        {
            Ok(result) => {
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for EyesMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Visual QA tools for agent-eyes. Tools: eyes_describe_dom (parse HTML into interactive elements), eyes_diff_screenshots (capture + pixel diff), eyes_vlm_caption (caption image with local LLaVA)."
                    .into(),
            ),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[tokio::test]
    async fn eyes_describe_dom_parses_simple_html() {
        let mcp = EyesMcp::new(Config::default());
        let params = DescribeDomParams {
            html: "<html><body><a href='/test'>click</a></body></html>".into(),
            max_elements: 100,
        };
        let result = mcp.eyes_describe_dom(params).await.unwrap();
        let text = result.content.iter()
            .filter_map(|c| if let RawContent::Text(t) = &c.raw { Some(&t.text[..]) } else { None })
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("\"tag\": \"a\""), "expected anchor tag in {text}");
        assert!(text.contains("click"), "expected link text in {text}");
    }

    #[tokio::test]
    async fn eyes_describe_dom_bare_html_includes_root() {
        let mcp = EyesMcp::new(Config::default());
        let params = DescribeDomParams {
            html: "<html></html>".into(),
            max_elements: 100,
        };
        let result = mcp.eyes_describe_dom(params).await.unwrap();
        let text = result.content.iter()
            .filter_map(|c| if let RawContent::Text(t) = &c.raw { Some(&t.text[..]) } else { None })
            .collect::<Vec<_>>()
            .join("");
        assert!(text.contains("\"tag\": \"html\""), "expected html root tag in {text}");
    }

    #[test]
    fn default_max_elements_is_5000() {
        assert_eq!(default_max_elements(), 5000);
    }

    #[test]
    fn default_diff_threshold_is_1_0() {
        assert_eq!(default_diff_threshold(), 1.0);
    }
}
