<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# nemo-flow

`nemo-flow` is the core Rust runtime crate for NeMo Flow.

It provides:

- hierarchical scopes and scope-local state
- tool and LLM lifecycle helpers
- guardrails and intercept chains
- subscribers and observability exporters
- plugin registration and activation primitives
- stream wrapping and typed runtime data structures

Use this crate when you want the Rust-first runtime surface. Pair it with
`nemo-flow-adaptive` when you need adaptive runtime behavior.

For project-level documentation, start with:

- the repo root `README.md`
- `docs/getting-started/rust.md`
- `docs/instrument-applications/about.md`
- `docs/integrate-frameworks/about.md`
- `docs/reference/api/index.md`
