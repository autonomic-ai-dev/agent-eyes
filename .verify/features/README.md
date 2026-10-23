# agent-eyes features

## Baseline preconditions

- Rust toolchain matching `Cargo.lock`.
- `capture` needs network access. `vlm` needs Ollama running, and the `candle` backend needs `--features vlm`.

## Driving conventions

- Unit tests run through `cargo test`. Logs go to `.verify/artifacts/<RUN_ID>/`.

## Proof standards

- Visual baselines are host-specific. A diff is only meaningful when the baseline and the run come from the same host.

## Feature index

| Feature file | Sub-features | Primary surface |
|---|---|---|
| `capture-and-diff.md` | 6 | `capture`, `diff`, `verify`, `describe`, `dom`, `vlm`, HTTP on 3105 |
