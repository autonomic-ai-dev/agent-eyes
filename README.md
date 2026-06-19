# agent-eyes

**Observability and Visual QA for AI agents — Headless Playwright testing, pixel-diff validation, and real-time telemetry dashboards.**

agent-eyes provides the visual and telemetry layer for the ecosystem. It bridges the gap between text-based LLMs and graphical user interfaces, allowing the agent to visually inspect UI changes while providing human developers with a LangSmith-style dashboard of the agent's internal state.

Rust is the optic nerve; Playwright is the retina.

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-eyes/master/scripts/install.sh | bash -s -- --global
agent-eyes serve --port 8080
```

**Dashboard is live immediately** — visit `http://localhost:8080` to see real-time agent trajectories.

---

## Why agent-eyes?

Text-based LLMs are fundamentally blind. 

1. **The UI Blindspot:** An agent can write perfect React code, but if a CSS `z-index` bug hides the button behind a modal, the agent has no idea.
2. **The Black Box:** When `agent-spine` runs a 50-node parallel workflow, watching JSON scroll in the terminal is impossible for humans to parse.
3. **Telemetry Sprawl:** Logs are scattered across `brain`, `spine`, `heart`, and `immune`.

**agent-eyes fixes this with a unified visual subsystem:**

| Problem | agent-eyes answer |
|---------|-------------------|
| "The agent claims the UI works, but it's broken" | **Visual QA** — uses headless Playwright to take screenshots of UI changes and perform pixel-diff testing against a golden master. |
| "I have no idea what the agent is doing" | **Observability Dashboard** — a local web dashboard that visualizes the `agent-spine` DAG execution, tool success rates, and token usage in real-time. |
| "Logs are scattered" | **Trace Logging** — structures stdout/stderr from all organs into a single, queryable SQLite telemetry stream. |

---

## Architectural Deep Dive

`agent-eyes` serves two distinct masters: the Agent (providing vision capabilities) and the Human (providing observability).

### 1. Vision Capabilities (For the Agent)
`agent-eyes` exposes an MCP tool called `take_screenshot`. 
- When an agent builds a web component, it triggers `agent-eyes` to boot a headless chromium instance, render the HTML/CSS, and capture a screenshot.
- It then passes the screenshot through a highly-quantized local Vision-Language Model (VLM) like LLaVA, translating the pixels back into a bounding-box JSON structure so the agent can "see" where elements are positioned.

### 2. The Local Observability Dashboard
No SaaS APIs. No sending telemetry to Datadog or LangSmith.
- `agent-eyes` ingests `nats.rs` events from `agent-nerves`.
- It powers a rich React/Next.js dashboard running on `localhost`.
- **Time-Travel Debugging:** Click on any node in a past workflow to see exactly what prompt was generated, what tools were called, and the raw API latency.

---

## Complete Setup (Copy & Paste)

### 1. Install the binary

```bash
curl -fsSL https://raw.githubusercontent.com/autonomic-ai-dev/agent-eyes/master/scripts/install.sh | bash -s -- --global
```

### 2. Configuration (`~/.agent_eyes/config.yaml`)

```yaml
telemetry:
  retention_days: 14
  port: 8080

vision:
  engine: playwright
  default_viewport: "1920x1080"
  local_vlm_fallback: true
```

### 3. Verify

```bash
agent-eyes version
agent-eyes health
```

---

## Commands

| Command | Description |
|---------|-------------|
| `agent-eyes serve` | Start the local observability dashboard on port 8080 |
| `agent-eyes diff <url1> <url2>` | Run an immediate pixel-diff test |
| `agent-eyes capture <url>` | Take a headless screenshot and save to disk |

---

## Development

```bash
cargo test --release -p agent-eyes
cargo build --release -p agent-eyes
```

## License
MIT
