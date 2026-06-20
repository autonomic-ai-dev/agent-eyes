# Changelog

## [Unreleased]

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
