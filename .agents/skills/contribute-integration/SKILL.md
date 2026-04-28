---
name: contribute-integration
description: Contribute a new or updated third-party framework integration for NeMo Flow
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---


# Contribute A Framework Integration

## Companion Guidance

Use `karpathy-guidelines` alongside this skill for implementation or review
work. Keep changes scoped, surface assumptions, and define focused validation
before editing.

Use this skill when contributing an integration with an upstream framework such
as LangChain, LangGraph, or another patched third-party project.

## Default Guidance

- keep NeMo Flow optional
- preserve the framework's original behavior when NeMo Flow is absent
- wrap tool and LLM paths at the correct framework boundary
- keep the tracked patch artifact minimal and reproducible

## Checklist

- [ ] integration pattern follows `docs/integrate-frameworks/adding-scopes.md`
- [ ] patch applies cleanly via `./scripts/apply-patches.sh --check`
- [ ] patch artifact regenerated if the local checkout changed
- [ ] relevant integration tests or smoke path pass
- [ ] docs updated if activation or usage changed

Use the root `./scripts/*.sh` commands in docs and contributor guidance. Their
implementations now live under `scripts/third-party/`.

## References

- `add-integration`
- `maintain-integration-patches`
- `docs/integrate-frameworks/about.md`
- `validate-change`
