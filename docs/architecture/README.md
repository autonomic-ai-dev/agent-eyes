# agent-eyes architecture documentation

## Design goals

agent-eyes gives AI agents the ability to "see" web pages and UIs. The key insight: **structure first, pixels second.** DOM parsing is fast, deterministic, and works without a browser. Screenshots and pixel diffs add the visual layer on top.

### Two processing paths

```
Path A: Structure (no browser, ~10ms)
  HTML file → DOM parser → headings/links/forms extraction → JSON output
  
Path B: Visual (requires browser, ~500ms-2s)
  URL → headless browser → screenshot PNG → pixel diff / VLM description
```

Path A is designed for every-turn use (agents can understand page structure without launching a browser). Path B is for CI and periodic regression checks.

### SQLite DOM index

```
agent-eyes dom index http://localhost:8765/page.html
  → Parses HTML, extracts all elements with attributes
  → Inserts into SQLite: { url, tag, id, class, text, attributes, parent_path }
  → Enables: "find the button with text 'Submit'" without re-parsing

DOM database: ~/.autonomic/memory/eyes_dom.db
```

The DOM index is persistent across agent sessions. Once a page is indexed, agents can query element locations without re-parsing the HTML.

### Local VLM (optional)

The VLM feature (`--features vlm`) adds local LLaVA inference via Candle:

```
agent-eyes vlm describe screenshot.png
  → LLaVA-1.5-7B via Candle (no GPU required, ~5s on Apple Silicon)
  → Returns: natural language description of the image
```

This is explicitly **local-only** — no image data ever leaves the machine.

### Key design decisions

| Decision | Rationale |
|----------|-----------|
| **Structure before screenshots** | DOM parsing is ~50x faster than browser capture. Use it as the default path. |
| **SQLite for DOM index** | Persistent element database survives agent restarts. No need to re-parse every turn. |
| **Local VLM, not cloud API** | Screenshots may contain sensitive UI data. LLaVA via Candle keeps everything on-device. |
| **Feature-gated VLM** | Not every install needs a 7B parameter vision model. `--features vlm` is opt-in. |

### Alternatives considered

| Option | Why rejected |
|--------|-------------|
| **Playwright/Puppeteer** | Heavy dependencies (Node.js, browser binaries). agent-eyes uses a lightweight Rust headless browser client. |
| **Cloud vision API (GPT-4V)** | Sends screenshots to third-party servers. LLaVA is slower but private. |
| **In-memory DOM only** | Loses indexed data on restart. SQLite persistence is cheap and valuable. |
