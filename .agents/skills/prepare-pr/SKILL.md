---
name: prepare-pr
description: Prepare a NeMo Flow branch for review with the right tests, docs, and contributor hygiene
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---


# Prepare A PR For NeMo Flow

## Companion Guidance

Use `karpathy-guidelines` alongside this skill for implementation or review
work. Keep changes scoped, surface assumptions, and define focused validation
before editing.

Use this skill at the end of a contributor or maintainer change before opening a
pull request.

## Checklist

- [ ] branch scope is coherent and reviewable
- [ ] relevant tests passed under `validate-change`
- [ ] changed files were formatted with the language-native formatter
- [ ] any Rust change ran `just test-rust`
- [ ] any Rust change ran `cargo fmt --all`
- [ ] any Rust change ran `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `crates/core` or `crates/adaptive` changes ran the full language matrix
- [ ] targeted `uv run pre-commit run --files <changed files...>` checks were used during iteration where useful
- [ ] `uv run pre-commit run --all-files` passed or issues are understood
- [ ] docs and examples updated for any public behavior changes
- [ ] dependent maintainer or consumer skills updated when code changes affected
      their APIs, bindings, commands, paths, packaging guidance, or best
      practices
- [ ] commit messages and PR summary explain what changed, why, and how it was tested
- [ ] breaking changes or renamed surfaces are called out explicitly

## PR Description Should Cover

- what changed
- why the change exists
- key implementation notes
- tests run
- any breaking behavior or migration notes

## References

- `CONTRIBUTING.md`
- `validate-change`
