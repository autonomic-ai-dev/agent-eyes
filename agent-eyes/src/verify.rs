use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::capture;
use crate::diff;

#[derive(Debug, Serialize)]
pub struct VerifyReport {
    pub passed: bool,
    pub diff_percent: f64,
    pub diff_pixels: u64,
    pub total_pixels: u64,
    pub baseline: PathBuf,
    pub capture: PathBuf,
    pub threshold_percent: f64,
}

pub fn baseline_dir() -> PathBuf {
    agent_body_core::organ_state_dir("eyes").join("baselines")
}

pub fn baseline_for_target(target: &str) -> PathBuf {
    let slug = target
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    baseline_dir().join(format!("{slug}.png"))
}

/// Capture target URL and compare against stored baseline (or seed baseline on first run).
pub async fn verify_ui(
    target: &str,
    baseline: Option<&Path>,
    threshold_percent: f64,
    update_baseline: bool,
) -> Result<VerifyReport> {
    std::fs::create_dir_all(baseline_dir())?;
    let baseline_path = baseline
        .map(Path::to_path_buf)
        .unwrap_or_else(|| baseline_for_target(target));
    let capture_path = baseline_dir().join("last_capture.png");

    capture::capture_url(target, &capture_path).await?;

    if !baseline_path.exists() || update_baseline {
        std::fs::copy(&capture_path, &baseline_path)
            .with_context(|| format!("seed baseline at {}", baseline_path.display()))?;
        return Ok(VerifyReport {
            passed: true,
            diff_percent: 0.0,
            diff_pixels: 0,
            total_pixels: 0,
            baseline: baseline_path,
            capture: capture_path,
            threshold_percent,
        });
    }

    let diff_path = baseline_dir().join("last_diff.png");
    let metrics = diff::pixel_diff_metrics(&baseline_path, &capture_path, &diff_path)?;
    let passed = metrics.diff_percent <= threshold_percent;

    Ok(VerifyReport {
        passed,
        diff_percent: metrics.diff_percent,
        diff_pixels: metrics.diff_pixels,
        total_pixels: metrics.total_pixels,
        baseline: baseline_path,
        capture: capture_path,
        threshold_percent,
    })
}
