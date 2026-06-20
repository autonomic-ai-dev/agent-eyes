# agent-eyes

**Observability and visual QA — URL capture, pixel diffing, and telemetry.**

agent-eyes captures web pages as images and compares them with pixel-perfect diffs for visual regression testing.

---

## Why agent-eyes?

| Problem | agent-eyes answer |
|---------|-------------------|
| "Did the UI change?" | **Pixel diff** — compare two screenshots, highlight differences |
| "I need to capture a page" | **URL capture** — download any URL to file |

## Commands

| Command | Description |
|---------|-------------|
| `agent-eyes serve` | Start daemon (future: telemetry) |
| `agent-eyes capture <url>` | Download a URL to a file |
| `agent-eyes diff <ref> <comp>` | Compare two images, output diff |
| `agent-eyes status` | Show config path |

---

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-eyes/master/scripts/install.sh | bash
```

## Development

```bash
cargo build --release -p agent-eyes
cargo test --release -p agent-eyes
```

## License

MIT
