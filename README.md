# agent-eyes — UI Observability and Visual QA

Agent context: [CONTEXT.md](CONTEXT.md) · Visual QA Guide: [https://autonomic-ai.dev/docs/visual-qa](https://autonomic-ai.dev/docs/visual-qa) · Architecture: [https://autonomic-ai.dev/docs/architecture](https://autonomic-ai.dev/docs/architecture).

**Cloud-Native role: Observability** (traces + visual state) — DOM indexing, headless structure extraction, and OpenTelemetry integrations.

`agent-eyes` provides **state extraction** for agent-observable UIs. Agents operating on web applications need to verify UI changes, but re-parsing a massive HTML payload on every LLM turn wastes context limits. `agent-eyes` solves this by aggressively indexing DOM state into local SQLite databases and broadcasting UI regressions via OpenTelemetry.

---

## Under the Hood: How it Works

### 1. Headless Structure Extraction
Instead of launching a massive headless browser (like Puppeteer/Playwright) just to verify a button exists, `agent-eyes` parses raw HTML strings natively in Rust. It rapidly extracts headings, links, forms, and interactive elements into a compressed JSON structure (`describe`) fast enough to run on every single agent turn.

### 2. DOM Indexing (SQLite)
When an agent is crawling a complex web app, `agent-eyes` indexes the precise element locations and hierarchy into a local SQLite database (`dom index`). The agent can then use fast SQL queries to find specific elements (e.g., "find all buttons with text 'Submit'") without needing to process the entire DOM in its context window.

### 3. OpenTelemetry Integration
When `agent-eyes` detects a visual UI regression (via pixel diffing or DOM comparison), it doesn't just print to the console. It broadcasts a structured event over the NATS JetStream bus and records an OpenTelemetry trace. This allows the Autonomic CI Dashboard to visualize exactly which LLM action broke the UI layout in real-time.

### 4. Local VLM (Zero-Data-Leak Vision)
If an agent needs to "look" at a generated screenshot, sending the image to OpenAI is a massive data leak for enterprise apps. `agent-eyes` sends the image to a **local Ollama server** running a vision model (default `qwen2.5vl:7b`), so the caption is generated on-device and the image never leaves the machine.

```mermaid
flowchart LR
    Input["URL / HTML / Image"] --> Capture["capture<br>Screenshot to PNG"]
    Input --> Describe["describe<br>DOM structure (no browser)"]
    Input --> Index["dom index<br>SQLite element DB"]

    Capture --> Diff["diff<br>Pixel comparison"]
    Capture --> VLM["vlm describe<br>Ollama vision model (local)"]

    Index --> Search["dom search<br>Element lookup by selector"]
```

---

## Standalone vs Integrated

| Mode | What you type | What happens |
|------|--------------|--------------|
| **Standalone** | `agent-eyes describe ./page.html` | Extract DOM structure: headings, links, forms |
| **Standalone** | `agent-eyes capture https://example.com` | Download URL to PNG file |
| **Standalone** | `agent-eyes dom index http://localhost:8765/page.html` | Index DOM into SQLite |
| **Standalone** | `agent-eyes diff before.png after.png` | Pixel diff with visual output |
| **Integrated** | HTTP daemon on `:3105` | Spine events (`eyes.captured`, `eyes.dom.indexed`) |
| **Integrated** | agent-spine | UI regression testing in workflow pipelines |

In standalone mode, eyes is a CLI visual QA tool. In integrated mode, it runs as a daemon that spine workflows query for automated UI verification.

---

## Why agent-eyes?

| Problem | agent-eyes answer |
|---------|-------------------|
| AI agents can't "see" the UI | **`capture` + `describe`** — structure analysis and screenshots |
| UI regressions go unnoticed in CI | **`diff`** — pixel comparison with diff image output |
| Re-parsing DOM every turn is wasteful | **`dom index`** — SQLite element lookup by URL, persistent across turns |
| Cloud vision sends screenshots off-device | **`vlm describe`** — local vision model via Ollama, no data leaves your machine |

---

## What you get

| Feature | Why use it |
|---------|------------|
| **Screenshot capture** | `capture <url>` — visual artifacts for QA and archiving |
| **Pixel diff** | `diff a.png b.png` — regression detection with visual diff output |
| **DOM indexing** | `dom index <url>` — persistent SQLite element database |
| **Structure extraction** | `describe <file>` — headings, links, forms without a browser |
| **Local VLM** | `vlm describe` — on-device image captions (requires a running Ollama server) |
| **HTTP daemon** | `serve` — spine and CI integration |

DOM database: `~/.autonomic/memory/eyes_dom.db`

---

## Commands

| Command | Description |
|---------|-------------|
| `agent-eyes capture <url>` | Download URL to PNG image file |
| `agent-eyes diff <a> <b>` | Pixel diff with diff image output |
| `agent-eyes describe <file>` | Page/file structure analysis (no browser) |
| `agent-eyes verify` | UI regression check against baseline |
| `agent-eyes dom index\|file\|stats\|search` | SQLite DOM index management |
| `agent-eyes vlm describe\|status` | Local vision model via Ollama (`candle` backend requires `--features vlm`) |
| `agent-eyes serve` | HTTP daemon on port 3105 |
| `agent-eyes serve-mcp` | Start MCP stdio server only (no HTTP daemon) |
| `agent-eyes status` | Show config, DOM stats, VLM state |

Global `--progress` (or `AGENT_PROGRESS=1`) enables structured ProgressTree CLI output.

---

## HTTP API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Daemon health |
| `POST` | `/capture` | Capture screenshot |
| `POST` | `/diff` | Pixel comparison |
| `POST` | `/dom/index` | Index DOM from URL |
| `GET` | `/dom/stats` | Indexed DOM statistics |
| `GET` | `/dom/search` | Search indexed elements |
| `GET` | `/vlm/status` | VLM model status |
| `POST` | `/vlm/describe` | Describe image via local VLM |

---

## Quick Install

### Homebrew (macOS — Recommended)

```bash
brew tap autonomic-ai-dev/tap
brew install agent-eyes

# Or install the full stack:
brew install autonomic-stack
```

### Direct Download / curl

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-eyes/master/scripts/install.sh | bash

# Or full stack via curl:
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-body/master/scripts/install-all-organs.sh | bash
```

Verify:
```bash
agent-eyes version
agent-eyes status
agent-eyes describe ./page.html
```

---

## Configuration

Section `[eyes]` in `~/.autonomic/config.toml` (default port **3105**).

```toml
[vlm]
enabled = true
backend = "ollama"
ollama_url = "http://127.0.0.1:11434"
ollama_model = "qwen2.5vl:7b"
```

- `backend = "ollama"` (default) calls a local Ollama server and needs no build feature. Start it with `ollama serve` and pull the model once with `ollama pull qwen2.5vl:7b`.
- `backend = "candle"` runs LLaVA in-process and needs `cargo build --release -p agent-eyes --features vlm`. It is limited to LLaVA 1.5 at the moment. LLaVA-NeXT fails inside candle 0.8.4, so use the Ollama backend for it.

---

## Local Setup

```bash
git clone https://github.com/autonomic-ai-dev/agent-eyes.git && cd agent-eyes
cargo build --release -p agent-eyes

# Serve a local HTML file, then index by URL:
python3 -m http.server 8765 &
agent-eyes dom index http://127.0.0.1:8765/page.html
```

---

## Architecture & Documentation

For complete architectural specifications, DOM SQLite schemas, and MCP tool contracts, visit the centralized documentation hub:
- **Visual QA & DOM Indexing Guide**: [https://autonomic-ai.dev/docs/visual-qa](https://autonomic-ai.dev/docs/visual-qa)
- **Ecosystem Architecture**: [https://autonomic-ai.dev/docs/architecture](https://autonomic-ai.dev/docs/architecture)

---

## Development

```bash
cargo test --release -p agent-eyes
```

---

## License

MIT
