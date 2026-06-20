use anyhow::Result;
use image::GenericImageView;
use std::path::Path;

pub async fn describe_target(target: &str) -> Result<()> {
    if target.starts_with("http://") || target.starts_with("https://") {
        describe_url(target).await
    } else if Path::new(target).exists() {
        describe_file(Path::new(target))
    } else {
        let with_scheme = format!("http://{}", target);
        describe_url(&with_scheme).await
    }
}

async fn describe_url(url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client.get(url).send().await?;
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let bytes = resp.bytes().await?;
    let size_kb = bytes.len() as f64 / 1024.0;

    println!("  Page Analysis for: {}", url);
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Content-Type: {}", content_type);
    println!("  Size:         {:.1} KB", size_kb);

    if content_type.starts_with("text/html") || content_type.contains("html") {
        let html = String::from_utf8_lossy(&bytes);
        analyze_html(&html)?;
    } else if content_type.starts_with("image/") {
        let temp_dir = std::env::temp_dir().join("agent-eyes");
        std::fs::create_dir_all(&temp_dir)?;
        let temp_path = temp_dir.join("describe_capture.png");
        std::fs::write(&temp_path, &bytes)?;
        analyze_image(&temp_path)?;
        std::fs::remove_file(&temp_path).ok();
    } else if content_type.contains("json") || content_type.contains("text") {
        let text = String::from_utf8_lossy(&bytes);
        println!("  Preview:");
        for line in text.lines().take(20) {
            if line.len() > 120 {
                println!("    {}", &line[..117]);
            } else {
                println!("    {}", line);
            }
        }
        if text.lines().count() > 20 {
            println!("    ... ({} more lines)", text.lines().count() - 20);
        }
    } else {
        println!("  (binary content, cannot display text preview)");
    }

    Ok(())
}

fn analyze_html(html: &str) -> Result<()> {
    let lower = html.to_lowercase();

    let title = extract_between(html, "<title", "</title>")
        .or_else(|| extract_between(html, "<title>", "</title>"))
        .unwrap_or("(no title)".to_string());
    println!("  Title:        {}", title);

    let h1_count = count_tags(html, "<h1");
    let h2_count = count_tags(html, "<h2");
    println!("  Headings:     {} h1, {} h2", h1_count, h2_count);

    let link_count = count_tags(html, "<a ");
    println!("  Links:        {}", link_count);

    let img_count = count_tags(html, "<img");
    println!("  Images:       {}", img_count);

    if lower.contains("react") || lower.contains("reactroot") {
        println!("  Framework:    React");
    } else if lower.contains("vue") {
        println!("  Framework:    Vue");
    } else if lower.contains("angular") {
        println!("  Framework:    Angular");
    } else if lower.contains("next.js") || lower.contains("nextjs") {
        println!("  Framework:    Next.js");
    }

    let contains_error = lower.contains("error") || lower.contains("500");
    let contains_not_found = lower.contains("404") || lower.contains("not found");
    let contains_welcome = lower.contains("welcome") || lower.contains("getting started");

    if contains_error || contains_not_found {
        println!("  Warning: Page may contain errors (error/404 detected)");
    }
    if contains_welcome {
        println!("  Note: Page appears to be a default/welcome page");
    }

    let stripped = html.replace(|c: char| c == '<' || c == '>', " ");
    let word_count = stripped.split_whitespace().count();
    println!("  Words:        ~{}", word_count);

    Ok(())
}

fn analyze_image(path: &Path) -> Result<()> {
    match image::open(path) {
        Ok(img) => {
            let (w, h) = img.dimensions();
            let color_type = img.color();
            let size_kb = std::fs::metadata(path)
                .map(|m| m.len() as f64 / 1024.0)
                .unwrap_or(0.0);

            println!("  Dimensions:   {} x {} px", w, h);
            println!("  File size:    {:.1} KB", size_kb);
            println!("  Color:        {:?}", color_type);
        }
        Err(_) => {
            println!("  (could not decode image)");
        }
    }
    Ok(())
}

fn describe_file(path: &Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let size_kb = std::fs::metadata(path)
        .map(|m| m.len() as f64 / 1024.0)
        .unwrap_or(0.0);

    println!("  File:         {}", path.display());
    println!("  Size:         {:.1} KB", size_kb);

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" => {
            analyze_image(path)?;
        }
        "html" | "htm" => {
            let content = std::fs::read_to_string(path)?;
            analyze_html(&content)?;
        }
        _ => {
            let content = std::fs::read_to_string(path);
            if let Ok(text) = content {
                let lines: Vec<&str> = text.lines().collect();
                println!("  Lines:        {}", lines.len());
                println!("  Preview:");
                for line in lines.iter().take(10) {
                    if line.len() > 120 {
                        println!("    {}", &line[..117]);
                    } else {
                        println!("    {}", line);
                    }
                }
                if lines.len() > 10 {
                    println!("    ... ({} more lines)", lines.len() - 10);
                }
            } else {
                println!("  (binary file)");
            }
        }
    }

    Ok(())
}

fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> Option<String> {
    let start_pos = text.find(start)?;
    let from_start = &text[start_pos + start.len()..];
    let end_pos = from_start.find('>')?;
    let after_tag = &from_start[end_pos + 1..];
    let content_end = after_tag.find(end)?;
    Some(after_tag[..content_end].trim().to_string())
}

fn count_tags(text: &str, tag: &str) -> usize {
    let lower = text.to_lowercase();
    let mut count = 0;
    let mut pos = 0;
    while let Some(found) = lower[pos..].find(tag) {
        count += 1;
        pos += found + 1;
    }
    count
}
