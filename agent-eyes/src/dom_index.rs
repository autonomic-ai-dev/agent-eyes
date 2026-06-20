//! Continuous DOM indexing into ~/.autonomic/memory/eyes_dom.db.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomIndexReport {
    pub url: String,
    pub title: Option<String>,
    pub elements_indexed: u64,
    pub pages_total: u64,
    pub db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomStats {
    pub db_path: String,
    pub pages: u64,
    pub elements: u64,
    pub last_indexed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomSearchHit {
    pub url: String,
    pub tag: String,
    pub element_id: Option<String>,
    pub class_name: Option<String>,
    pub text: String,
    pub path: String,
}

pub fn db_path() -> PathBuf {
    std::env::var("AUTONOMIC_EYES_DOM_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| agent_body_core::memory_dir().join("eyes_dom.db"))
}

pub fn open_db() -> Result<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path).with_context(|| format!("open {}", path.display()))?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY,
            url TEXT NOT NULL UNIQUE,
            title TEXT,
            indexed_at INTEGER NOT NULL,
            element_count INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS dom_elements (
            id INTEGER PRIMARY KEY,
            page_id INTEGER NOT NULL,
            tag TEXT NOT NULL,
            element_id TEXT,
            class_name TEXT,
            text TEXT,
            path TEXT NOT NULL,
            FOREIGN KEY(page_id) REFERENCES pages(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_dom_tag ON dom_elements(tag);
        CREATE INDEX IF NOT EXISTS idx_dom_text ON dom_elements(text);
        "#,
    )?;
    Ok(())
}

pub async fn index_url(url: &str, max_elements: usize) -> Result<DomIndexReport> {
    let normalized = normalize_url(url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let resp = client.get(&normalized).send().await?;
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    anyhow::ensure!(
        content_type.contains("html") || content_type.is_empty(),
        "URL did not return HTML (content-type: {content_type})"
    );
    let html = resp.text().await?;
    index_html(&normalized, &html, max_elements)
}

pub fn index_html(url: &str, html: &str, max_elements: usize) -> Result<DomIndexReport> {
    let title = extract_title(html);
    let elements = parse_dom_elements(html, max_elements);
    let conn = open_db()?;
    let now = chrono::Utc::now().timestamp_millis();
    let count = elements.len() as u64;

    conn.execute(
        "DELETE FROM dom_elements WHERE page_id IN (SELECT id FROM pages WHERE url = ?1)",
        params![url],
    )?;
    conn.execute("DELETE FROM pages WHERE url = ?1", params![url])?;
    conn.execute(
        "INSERT INTO pages (url, title, indexed_at, element_count) VALUES (?1, ?2, ?3, ?4)",
        params![url, title, now, count],
    )?;
    let page_id = conn.last_insert_rowid();
    for el in elements {
        conn.execute(
            "INSERT INTO dom_elements (page_id, tag, element_id, class_name, text, path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                page_id,
                el.tag,
                el.element_id,
                el.class_name,
                el.text,
                el.path
            ],
        )?;
    }

    let pages_total: u64 = conn.query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))?;

    Ok(DomIndexReport {
        url: url.to_string(),
        title,
        elements_indexed: count,
        pages_total,
        db_path: db_path().display().to_string(),
    })
}

pub fn load_stats() -> Result<DomStats> {
    let conn = open_db()?;
    let pages: u64 = conn.query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))?;
    let elements: u64 =
        conn.query_row("SELECT COUNT(*) FROM dom_elements", [], |row| row.get(0))?;
    let last_indexed_at: Option<i64> = conn
        .query_row("SELECT MAX(indexed_at) FROM pages", [], |row| row.get(0))
        .ok();
    Ok(DomStats {
        db_path: db_path().display().to_string(),
        pages,
        elements,
        last_indexed_at,
    })
}

pub fn search(query: &str, limit: u32) -> Result<Vec<DomSearchHit>> {
    let conn = open_db()?;
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT p.url, e.tag, e.element_id, e.class_name, e.text, e.path
        FROM dom_elements e
        JOIN pages p ON p.id = e.page_id
        WHERE e.text LIKE ?1 OR e.tag LIKE ?1 OR e.element_id LIKE ?1 OR e.class_name LIKE ?1
        ORDER BY p.indexed_at DESC
        LIMIT ?2
        "#,
    )?;
    let rows = stmt.query_map(params![pattern, limit], |row| {
        Ok(DomSearchHit {
            url: row.get(0)?,
            tag: row.get(1)?,
            element_id: row.get(2)?,
            class_name: row.get(3)?,
            text: row.get(4)?,
            path: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[derive(Debug, Clone)]
struct ParsedElement {
    tag: String,
    element_id: Option<String>,
    class_name: Option<String>,
    text: String,
    path: String,
}

fn parse_dom_elements(html: &str, max_elements: usize) -> Vec<ParsedElement> {
    let mut out = Vec::new();
    let mut path_stack: Vec<(String, usize)> = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() && out.len() < max_elements {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let start = i;
        let end = match html[start..].find('>') {
            Some(off) => start + off,
            None => break,
        };
        let tag_slice = &html[start + 1..end];
        if tag_slice.starts_with('!') || tag_slice.starts_with('?') || tag_slice.starts_with('/') {
            if tag_slice.starts_with('/') {
                let name = tag_slice
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                pop_tag(&mut path_stack, &name);
            }
            i = end + 1;
            continue;
        }

        let self_closing = tag_slice.ends_with('/') || is_void_tag(tag_slice);
        let tag_name = tag_slice
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_lowercase();
        if tag_name.is_empty() || tag_name == "script" || tag_name == "style" {
            i = end + 1;
            continue;
        }

        let attrs = parse_attrs(tag_slice);
        let idx = increment_tag(&mut path_stack, &tag_name);
        let path = format_path(&path_stack);
        let text = extract_inner_text(
            html,
            end + 1,
            &tag_name,
            max_elements.saturating_sub(out.len()),
        );

        out.push(ParsedElement {
            tag: tag_name.clone(),
            element_id: attrs.id,
            class_name: attrs.class,
            text: truncate_text(text, 240),
            path,
        });

        if !self_closing {
            path_stack.push((tag_name, idx));
        }
        i = end + 1;
    }

    out
}

#[derive(Default)]
struct Attrs {
    id: Option<String>,
    class: Option<String>,
}

fn parse_attrs(tag_slice: &str) -> Attrs {
    let lower = tag_slice.to_lowercase();
    Attrs {
        id: extract_attr(&lower, "id"),
        class: extract_attr(&lower, "class"),
    }
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = rest[1..].find(quote)? + 1;
    Some(rest[1..end].trim().to_string())
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let after = &html[start..];
    let gt = after.find('>')? + 1;
    let end = after[gt..].find("</title>")?;
    Some(after[gt..gt + end].trim().to_string())
}

fn extract_inner_text(html: &str, from: usize, tag: &str, max_chars: usize) -> String {
    let close = format!("</{tag}>");
    let slice = html.get(from..).unwrap_or("");
    let end = slice
        .to_lowercase()
        .find(&close)
        .unwrap_or(max_chars.min(slice.len()));
    let raw = &slice[..end.min(slice.len())];
    strip_tags(raw)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_tags(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn increment_tag(stack: &mut Vec<(String, usize)>, tag: &str) -> usize {
    if let Some((last_tag, count)) = stack.last_mut() {
        if last_tag == tag {
            *count += 1;
            return *count;
        }
    }
    1
}

fn pop_tag(stack: &mut Vec<(String, usize)>, tag: &str) {
    if let Some(pos) = stack.iter().rposition(|(t, _)| t == tag) {
        stack.truncate(pos);
    }
}

fn format_path(stack: &[(String, usize)]) -> String {
    if stack.is_empty() {
        "root".into()
    } else {
        stack
            .iter()
            .map(|(tag, idx)| format!("{tag}[{idx}]"))
            .collect::<Vec<_>>()
            .join(">")
    }
}

fn truncate_text(text: String, max: usize) -> String {
    if text.chars().count() <= max {
        text
    } else {
        text.chars().take(max).collect::<String>()
    }
}

fn is_void_tag(tag_slice: &str) -> bool {
    matches!(
        tag_slice
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_lowercase()
            .as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{url}")
    }
}

pub fn index_file(path: &Path, max_elements: usize) -> Result<DomIndexReport> {
    let html = std::fs::read_to_string(path)
        .with_context(|| format!("read html file {}", path.display()))?;
    let url = format!("file://{}", path.display());
    index_html(&url, &html, max_elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn indexes_sample_html() {
        let dir = TempDir::new().unwrap();
        std::env::set_var("AUTONOMIC_EYES_DOM_DB", dir.path().join("eyes_dom.db"));
        let html = r#"<!doctype html><html><head><title>Demo</title></head><body><div id="main" class="hero"><a href="/">Home</a></div></body></html>"#;
        let report = index_html("http://localhost/demo", html, 100).unwrap();
        assert!(report.elements_indexed >= 3);
        let hits = search("Home", 10).unwrap();
        assert!(!hits.is_empty());
        std::env::remove_var("AUTONOMIC_EYES_DOM_DB");
    }
}
