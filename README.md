# agent-eyes

`agent-eyes` checks web UIs without launching a browser. It captures screenshots, compares them, indexes page elements into SQLite, and describes images with a local vision model.

Docs: [autonomic-ai-dev.vercel.app](https://autonomic-ai-dev.vercel.app)

## Quick start

```bash
brew tap autonomic-ai-dev/tap && brew install agent-eyes
agent-eyes describe ./page.html
agent-eyes capture https://example.com
```

## Commands

| Command | What it does |
|---------|--------------|
| `agent-eyes capture <url>` | Saves a URL as a PNG |
| `agent-eyes diff <a> <b>` | Compares two images and writes a diff image |
| `agent-eyes describe <file>` | Lists headings, links, and forms in a page, without a browser |
| `agent-eyes verify` | Checks the UI against a baseline |
| `agent-eyes dom index\|file\|stats\|search` | Manages the SQLite element index |
| `agent-eyes vlm describe\|status` | Describes an image with a local vision model |
| `agent-eyes serve` | Runs the HTTP daemon on port 3105 |
| `agent-eyes serve-mcp` | Runs the MCP stdio server only |
| `agent-eyes status` | Shows the config, DOM index stats, and VLM state |

## HTTP API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Daemon health |
| `POST` | `/capture` | Captures a screenshot |
| `POST` | `/diff` | Compares two images |
| `POST` | `/dom/index` | Indexes the DOM of a URL |
| `GET` | `/dom/stats` | Indexed DOM statistics |
| `GET` | `/dom/search` | Searches indexed elements |
| `GET` | `/vlm/status` | VLM model status |
| `POST` | `/vlm/describe` | Describes an image with the local VLM |

## Vision model

Image descriptions run on a local model. The default backend is Ollama at `http://127.0.0.1:11434` with `qwen2.5vl:7b`. Run `ollama serve` and `ollama pull qwen2.5vl:7b` once. The in-process `candle` backend needs a build with `--features vlm`, and only LLaVA 1.5 works with it.

## Configuration

Settings live in the `[eyes]` and `[vlm]` sections of `~/.autonomic/config.toml`. The default port is 3105. The DOM index is stored at `~/.autonomic/memory/eyes_dom.db`.

## Develop

```bash
cargo test -p agent-eyes
cargo clippy -p agent-eyes -- -D warnings
```

Feature map and knowledge base: `.verify/features/` and `.verify/knowledge-base/`.
