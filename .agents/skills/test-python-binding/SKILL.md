---
name: test-python-binding
description: Build and test the NeMo Flow Python binding; use this for python/nemo_flow or crates/python changes
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---


# Build And Test Python Binding

## Companion Guidance

Use `karpathy-guidelines` alongside this skill for implementation or review
work. Keep changes scoped, surface assumptions, and define focused validation
before editing.

Use this skill when the change is primarily in `python/nemo_flow`,
`python/tests`, `crates/python`, or Python-facing docs/examples.

## Default Path

1. Format changed Python wrapper and test files with `uv run ruff format python`.
2. Run focused `pytest` first when you know the affected area.
3. Run the full Python suite with `just test-python` before review.
4. If any Rust files changed as part of the Python work, also run
   `cargo fmt --all`, `just test-rust`, and
   `cargo clippy --workspace --all-targets -- -D warnings`.
5. Use `just build-python` when you want an explicit build-only pass.
6. If the native Rust bridge changed, add the Rust crate tests for
   `nemo-flow-python`.

## Common Commands

```bash
# Focused test loop
uv run pytest -k "<pattern>"

# Format Python files
uv run ruff format python

# Full Python suite
just test-python

# Required when the Python change also touched Rust code
cargo fmt --all
just test-rust
cargo clippy --workspace --all-targets -- -D warnings

# Rebuild the editable package plus native extension
just build-python

# Native extension crate when crates/python changed
cargo test -p nemo-flow-python
```

## When To Escalate

- If `crates/core`, `crates/adaptive`, or shared runtime semantics changed,
  also use `validate-change`.
- If the change is actually about docs only, prefer `contribute-docs`
  plus targeted command checks.

## References

- `pyproject.toml`
- `crates/python/Cargo.toml`
- `crates/python/README.md`
- `python/nemo_flow/README.md`
- `docs/getting-started/python.md`
- `docs/contribute/testing-and-docs.md`
- `validate-change`
