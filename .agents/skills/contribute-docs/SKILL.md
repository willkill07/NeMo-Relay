---
name: contribute-docs
description: Contribute documentation or example changes that stay aligned with NeMo Flow public behavior
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---


# Contribute Docs Or Examples

## Companion Guidance

Use `karpathy-guidelines` alongside this skill for implementation or review
work. Keep changes scoped, surface assumptions, and define focused validation
before editing.

Use this skill for docs-only or example-heavy changes.

## Rules

- prefer the documented public API, not internal shortcuts
- keep package names, repo references, and build commands current
- update entry-point docs when examples or reading paths change
- keep release-process and release-notes guidance in repo-maintainer docs such as
  `RELEASING.md`, not as user-facing docs pages or `CHANGELOG.md`
- keep stable user-facing wrappers at `scripts/` root in docs and examples;
  only point at namespaced helper paths when documenting internal maintenance
  work

## Checklist

- [ ] `README.md` or `docs/index.md` updated when entry points changed
- [ ] relevant getting-started or reference docs updated
- [ ] example commands still match current package names and paths
- [ ] relevant package or crate `README.md` files updated when examples or binding guidance changed
- [ ] release-policy docs still point to GitHub Releases as the only release-history source of truth
- [ ] run `just docs` when the docs site changed; `./scripts/build-docs.sh html` remains the compatibility wrapper

## References

- `CONTRIBUTING.md`
- `RELEASING.md`
- `docs/contribute/testing-and-docs.md`
- `review-doc-style`
