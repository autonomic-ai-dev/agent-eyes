# capture-and-diff

Summary: agent-eyes captures web pages to PNG, compares two images pixel by pixel, checks a UI against a stored baseline, analyzes page structure without a browser, indexes DOM elements into SQLite for search, and describes images with a local vision model.

## Sub-features

- `capture-png`: Saves a URL as a PNG file.
  - Intent: Create a visual record of a page.
  - Behaviour: `agent-eyes capture <url>` and `POST /capture`.
  - Outcome: A PNG file at the chosen path.
- `pixel-diff`: Compares two images and writes a diff image.
  - Intent: Measure visual change as a number and a picture.
  - Behaviour: `agent-eyes diff <a> <b>` and `POST /diff`.
  - Outcome: A difference measure and a diff image.
- `ui-regression-verify`: Checks the current UI against a baseline.
  - Intent: Fail a run when the UI drifts.
  - Behaviour: `agent-eyes verify` compares against the stored baseline.
  - Outcome: Pass when within threshold, fail with the diff otherwise.
- `page-describe`: Analyzes page or file structure without a browser.
  - Behaviour: `agent-eyes describe <file>`, `POST /dom/index` for indexing.
  - Outcome: A structure report.
- `dom-index-search`: Indexes DOM elements into SQLite and searches them.
  - Behaviour: `agent-eyes dom index|file|stats|search`, `GET /dom/search`.
  - Outcome: Matching elements for a query.
- `vlm-describe`: Describes an image with a local vision model through Ollama.
  - Behaviour: `agent-eyes vlm describe|status`, `GET /vlm/status`, `POST /vlm/describe`. The `candle` backend needs `--features vlm`.
  - Outcome: A text description or a status with the model state.

## Code

- `agent-eyes`

## How to get to it (user POV)

- CLI: `agent-eyes capture`, `diff`, `describe`, `verify`, `dom`, `vlm`, `serve`, `serve-mcp`, `status`
- HTTP: `GET /health`, `POST /capture`, `POST /diff`, `POST /dom/index`, `GET /dom/stats`, `GET /dom/search`, `GET /vlm/status`, `POST /vlm/describe` on `http://localhost:3105` (all routes and the port are confirmed in `agent-eyes/src/serve.rs` and `src/config.rs`)
- MCP: `eyes_describe_dom`, `eyes_diff_screenshots`, `eyes_vlm_caption` through agent-body

## Verification Suites (Targeted Test Commands)

- **Unit Tests:** `cargo test` (whole crate, 18 tests in the baseline)
- **Functional API Tests (Hurl):** not yet written
- **Allowlisted Verification:** none yet. This crate has no `.verify/` harness.
- **Preconditions:** Rust toolchain. `capture` needs network access. `vlm` needs Ollama running.

### Verification Steps

1. **eyes-unit-tests** (sub-features: all)
   - Request: `cargo test`
   - Why: Proves the diff, DOM index, and describe code paths.
   - Expected: 18 passed, 0 failed (baseline).
   - Impact: A failure means visual or DOM logic changed. Fix the cause.

## Gotchas

- `capture-png` and `ui-regression-verify` depend on a live URL or a stored baseline. A baseline captured on another machine can differ by font rendering, so a diff is only meaningful on the same host.
- `vlm-describe` is gated behind a cargo feature for the `candle` backend. The default build does not include it.
- No test in this pass covers the HTTP routes, so the port 3105 contract is unproven.
- This crate has no `.verify/` harness.
