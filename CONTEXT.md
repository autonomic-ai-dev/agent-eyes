# agent-eyes — Context for agents

UI observability and visual QA for Autonomic. Part of [agent-body](https://github.com/autonomic-ai-dev/agent-body).

| Field | Value |
|-------|-------|
| Role | Observability / visual state |
| Binary | `agent-eyes` |
| HTTP | `:3105` |
| Version | v1.0.0-beta.2 |
| Docs Hub | [https://autonomic-ai.dev/docs/visual-qa](https://autonomic-ai.dev/docs/visual-qa) · [Architecture](https://autonomic-ai.dev/docs/architecture) |

## When to use

DOM describe/index without a full browser; pixel diff; optional local VLM captions. Prefer over dumping raw HTML into the LLM context.

## Key verbs

- `describe` / `dom index` / `dom search`
- `capture` / `diff`
- `vlm describe` (local)

## Verification

Visual gates should use allowlisted commands, e.g.:

```bash
agent-brain verify --source orchestrator -- agent-eyes verify localhost:3000
```

## CI

Path filter on `agent-eyes/**` + Cargo/Makefile/scripts/`.github`. Docs-only PRs skip compose.

## Related

Operator usage: [https://autonomic-ai.dev/docs/visual-qa](https://autonomic-ai.dev/docs/visual-qa) · Architecture: [https://autonomic-ai.dev/docs/architecture](https://autonomic-ai.dev/docs/architecture) · Spine (UI QA nodes) · mouth (summaries) · brain (verify)
