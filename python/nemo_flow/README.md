<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# NeMo Flow Python Package

This directory contains the public Python package source for `nemo-flow`.

Use the repository root `pyproject.toml` and `uv sync` for local development,
tests, and docs builds. The compiled extension is exposed as `nemo_flow._native`
and the public package surface lives under:

- `nemo_flow.scope`
- `nemo_flow.tools`
- `nemo_flow.llm`
- `nemo_flow.guardrails`
- `nemo_flow.intercepts`
- `nemo_flow.subscribers`
- `nemo_flow.plugin`
- `nemo_flow.adaptive`
- `nemo_flow.typed`
- `nemo_flow.codecs`

Primary user docs live in `docs/getting-started/python.md`,
`docs/instrument-applications/about.md`, `docs/integrate-frameworks/about.md`,
and `docs/reference/api/python/index.md`.
