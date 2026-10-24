# Capture and diff

## What it does

agent-eyes captures web pages to PNG, compares two images pixel by pixel, checks a UI against a stored baseline, analyzes page structure without a browser, indexes DOM elements into SQLite for search, and describes images with a local vision model.

## How it works

1. **Capture.** `agent-eyes capture <url>` and `POST /capture` save a page as a PNG.
2. **Pixel diff.** `agent-eyes diff <a> <b>` and `POST /diff` report a difference measure and write a diff image.
3. **UI regression.** `agent-eyes verify` compares the current UI with the stored baseline. It passes within the threshold and fails with the diff otherwise.
4. **Describe.** `agent-eyes describe <file>` reports page or file structure without a browser.
5. **DOM index and search.** `POST /dom/index` indexes elements into SQLite. `GET /dom/stats` reports counts, and `GET /dom/search` finds matching elements.
6. **Vision.** `agent-eyes vlm describe` and `POST /vlm/describe` describe an image through a local model. `GET /vlm/status` reports the model state. The `candle` backend needs the `vlm` cargo feature.

## Where the code is

- Routes: `agent-eyes/agent-eyes/src/serve.rs` lines 47 to 54
- Port 3105 and config: `agent-eyes/src/config.rs`
- DOM index: `dom_index` module, SQLite-backed
- MCP tools: `eyes_describe_dom`, `eyes_diff_screenshots`, `eyes_vlm_caption`

## How to use it

- `agent-eyes capture https://example.com --out page.png`
- `agent-eyes diff before.png after.png`
- `agent-eyes verify` in CI to fail on visual drift
- `agent-eyes vlm describe image.png` with Ollama running

## How it is verified

- `cargo test` for the whole crate (18 tests in the recorded baseline): diff, DOM index, and describe code paths
- `route_drift.sh`: the README route table, including `/dom/stats`, matches source

## Known limits

- Capture and UI regression depend on a live URL or a stored baseline. A baseline from another machine can differ by font rendering, so a diff is only meaningful on the same host.
- The vision backend is behind a cargo feature. The default build does not include it.
- No test covers the HTTP routes, so the port 3105 contract is unproven.
- This crate has no `.verify/` allowlist yet. Its local uncommitted VLM prompt change is not part of this entry.

## Related

- Feature map: `agent-eyes/.verify/features/capture-and-diff.md`
