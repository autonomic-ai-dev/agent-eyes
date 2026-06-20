use anyhow::Result;
use std::path::Path;

pub async fn capture_url(url: &str, output: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let bytes = resp.bytes().await?;
    std::fs::write(output, &bytes)?;
    println!("Downloaded {} ({} bytes) to {}", url, bytes.len(), output.display());
    Ok(())
}
