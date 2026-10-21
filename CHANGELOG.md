# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

### Added
- **Ollama VLM backend** — `[vlm] backend = "ollama"` (default target) sends screenshots to a local Ollama server. Settings: `ollama_url`, `ollama_model` (default `qwen2.5vl:7b`).
- **Real screenshot capture** — `capture` uses Playwright (`npx playwright@1.55.0 screenshot`) and checks the PNG signature.

### Changed
- `vlm` backend is selected with `backend = "candle"`; LLaVA-NeXT is not supported there because candle 0.8.4 fails in its multi-patch encode.

## [1.0.0-beta.3.1] - 2026-10-18

### Fixed
- **Tracing Stdio Leakage**: Routed tracing subscriber logs explicitly to `stderr` in `main.rs` to maintain clean MCP/JSON-RPC stdio pipes.
- **Test Concurrency Isolation**: Enhanced test environment isolation to prevent parallel test races.

### Changed
- Bumped version to `v1.0.0-beta.3.1`.

## [1.0.0-beta.3] - 2026-10-17

### Changed
- Bumped version to `v1.0.0-beta.3` and updated `agent-body-core` dependency to `v1.0.0-beta.3`.
- Relocated comprehensive `CONTEXT.md` to repository root and aligned documentation hub links.

## [1.0.0-beta.2] - 2026-10-16

### Added
- Official Homebrew tap formula support (`brew install agent-eyes`).

### Changed
- Consolidated architecture documentation into centralized ecosystem hub.

## [1.0.0-beta.1] - 2026-10-16

### Changed
- Unified ecosystem versioning to 1.0.0-beta.1
- Pinned `agent-body-core` dependency to `v1.0.0-beta.1`
- Preserved CI Developer ID signatures in `scripts/install.sh` (clears quarantine without re-signing)
- Dynamic GitHub pre-release flag in release workflows for beta/rc/alpha tags
- CI strictly arm64 macOS native runners with Developer ID code signing
- Aligned documentation and CI path filters with agent-brain

## [0.9.0] - 2026-07-18

### Added

- **Phase 9** `phash_cascade` precision module

## [0.7.8] - 2026-06-27

### Changed

- **agent-body-core dependency** — updated tag to `v0.5.14` for ecosystem alignment

## [0.7.7] - 2026-06-27

### Added

- **`serve-mcp` subcommand** — starts the MCP stdio server only (no HTTP daemon). Used by `agent-body serve-mcp` gateway for persistent tool aggregation.
- **Integration tests** — `ServeMcp` CLI parsing, `EyesMcp` tool handler response validation (DOM describe, diff screenshots)

## [0.7.6] - 2026-06-27

### Added

- Dual-mode MCP server with organ tool naming convention (`eyes_describe_dom`, `eyes_diff_screenshots`, `eyes_vlm_caption`)

## [0.7.5] - 2026-06-23

### Added

- Global `--progress` CLI flag for structured ProgressTree output (also `AGENT_PROGRESS=1`)

## [0.7.4] - 2026-06-21

### Added

- `agent-eyes update [--force]` — self-update subcommand that checks GitHub releases, compares versions, and downloads the latest binary

## [0.7.3] - 2026-06-21

### Added

- `agent-eyes log <name> [--follow] [--list]` — read daemon logs from the supervisor log directory

## [0.7.2] - 2026-06-21

### Fixed

- agent-spine registration is now non-fatal — daemon starts even without spine available

## [0.7.1] - 2026-06-20

### Added

- `--version` CLI flag (`fbe99b0`)
- Mermaid architecture charts in README (`8544d2c`)

### Changed

- Professional README with standalone and integrated usage (`59c7861`)

## [0.7.0] - 2026-06-20

### Added

- **Native local VLM** — LLaVA-1.5 via HuggingFace `candle` (`--features vlm`)
- **`agent-eyes vlm describe|status`** CLI and HTTP `/vlm/describe`, `/vlm/status`
- Config section `[vlm]` — `model_id`, `model_dir`, `max_new_tokens`, `temperature`, `cpu`
- Publishes `eyes.vlm.described` to agent-spine on successful caption

## [0.6.0] - 2026-06-20

### Added

- **Continuous DOM indexing** — SQLite store at `~/.autonomic/memory/eyes_dom.db`
- **`agent-eyes dom index|file|stats|search`** CLI and HTTP `/dom/index`, `/dom/stats`, `/dom/search`
- Publishes `eyes.dom.indexed` to agent-spine on index

## [0.5.0] - 2026-06-20

### Added

- **Visual UI verification** — `verify localhost:3000` pixel-diff against baselines before autonomous dataset loops

## [0.4.0] - 2026-06-20

### Added

- **Unified config** — loads from `~/.autonomic/config.toml` via `agent-body-core::organ_config::load("eyes")`

### Changed

- Version bumped from `0.3.0` to `0.4.0`

## [0.3.0] - 2026-06-20

### Added

- **Page description CLI** — `agent-eyes describe <url>` downloads and analyzes a web page, extracting title, headings, links, images, framework detection, and content statistics
- **Image analysis** — `agent-eyes describe <image>` returns dimensions, file size, and color type
- **File analysis** — `agent-eyes describe <file>` handles HTML, images, and text files with preview

### Changed

- Version bumped from `0.2.0` to `0.3.0`

## [0.2.0] - 2026-06-20

### Added

- **HTTP daemon** — `agent-eyes serve` now starts an axum HTTP server with `/health`, `/capture`, and `/diff` endpoints
- **Agent-spine integration** — registers with agent-spine event bus on startup, heartbeats every 30s, publishes `eyes.captured` and `eyes.diffed` events
- **Config extended** — `server.port` (default 3105) and `spine.url` (default `http://localhost:3100`) settings

### Changed

- Version bumped from `0.1.0` to `0.2.0`

## [0.1.0] - 2026-06-20

### Added

- **Initial project scaffold** — workspace, crate, config
- **URL capture** — downloads a URL and saves to file
- **Pixel diff** — compares two images, highlights differences in red
- **CLI** — `agent-eyes serve`, `capture <url>`, `diff <ref> <comp>`, `status`
- **CI pipeline** — test + build + release workflows
