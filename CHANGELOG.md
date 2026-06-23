# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
