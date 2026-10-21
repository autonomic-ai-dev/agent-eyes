use std::path::Path;

use anyhow::{bail, Context, Result};
use tokio::process::Command;

const PNG_SIGNATURE: &[u8] = &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
const PLAYWRIGHT_ARGS: &[&str] = &[
    "-y",
    "playwright@1.55.0",
    "screenshot",
    "--full-page",
    "--wait-for-timeout",
    "1000",
];

pub async fn capture_url(url: &str, output: &Path) -> Result<()> {
    run_screenshot("npx", PLAYWRIGHT_ARGS, url, output).await?;
    println!("Captured {} to {}", url, output.display());
    Ok(())
}

async fn run_screenshot(program: &str, base_args: &[&str], url: &str, output: &Path) -> Result<()> {
    let result = Command::new(program)
        .args(base_args)
        .arg(url)
        .arg(output)
        .output()
        .await
        .with_context(|| {
            format!("failed to start {program}; install Node.js and run `npx playwright install chromium`")
        })?;
    if !result.status.success() {
        bail!(
            "screenshot failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        );
    }
    let written = std::fs::read(output).context("screenshot file was not written")?;
    if !written.starts_with(PNG_SIGNATURE) {
        bail!("screenshot output is not a PNG image");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn fake_tool(name: &str, body: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("eyes-capture-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("fake-screenshot.sh");
        std::fs::write(&script, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        script
    }

    #[tokio::test]
    async fn accepts_output_that_is_a_png() {
        let tool = fake_tool("png", r#"printf '\211PNG\r\n\032\nrest' > "$2""#);
        let out = std::env::temp_dir().join(format!("eyes-shot-png-{}.png", std::process::id()));
        run_screenshot(tool.to_str().unwrap(), &[], "https://example.com", &out)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn rejects_output_that_is_html_not_an_image() {
        let tool = fake_tool("html", r#"printf '<!doctype html>' > "$2""#);
        let out = std::env::temp_dir().join(format!("eyes-shot-html-{}.png", std::process::id()));
        let err = run_screenshot(tool.to_str().unwrap(), &[], "https://example.com", &out)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not a PNG"));
    }

    #[tokio::test]
    async fn missing_playwright_explains_how_to_install_it() {
        let out = std::env::temp_dir().join("eyes-shot-missing.png");
        let err = run_screenshot(
            "/nonexistent/playwright-binary",
            &[],
            "https://example.com",
            &out,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("playwright install chromium"));
    }
}
