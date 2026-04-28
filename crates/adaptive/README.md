<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# nemo-flow-adaptive

`nemo-flow-adaptive` is the adaptive companion crate for `nemo-flow`.

It also owns the canonical ACG Rust module tree at `nemo_flow_adaptive::acg`,
implemented in `src/acg/`.

It provides:

- adaptive configuration types
- built-in adaptive plugin components
- telemetry subscribers and hint injection helpers
- in-memory and optional Redis-backed state
- learning and plan-selection primitives built on top of the core runtime
- the canonical `nemo_flow_adaptive::acg` module surface for PromptIR,
  provider plugins, stability analysis, and cache telemetry normalization

Use this crate when you want adaptive behavior on top of the generic NeMo Flow
plugin system.

## What It Provides

- `AdaptiveConfig` as the canonical config contract for the top-level `adaptive` plugin component
- typed helper configs for adaptive telemetry, hints, and tool parallelism sections
- `ComponentSpec` plus `register_adaptive_component()` for core-plugin host integration
- storage backends for in-memory and Redis-backed persistence

The ACG module does not rename `AdaptiveConfig.acg`, change runtime defaults, or
alter the persisted Redis key names used for ACG observations and stability.

## Feature Flags

The table below explains the feature flags that shape this crate build.

| Feature | Purpose |
|---------|---------|
| `redis-backend` | Enables the Redis-backed storage implementation |

The Redis backend is optional. Builds without `redis-backend` still support the
in-memory backend and the rest of the adaptive pipeline.

## Build

Use the command below when you need to build this package directly.

```bash
# Default build (in-memory backend only)
cargo build -p nemo-flow-adaptive

# Build with Redis backend support
cargo build -p nemo-flow-adaptive --features redis-backend
```

## Test

Use the command below when you need to test this package directly.

```bash
# In-memory adaptive tests
cargo test -p nemo-flow-adaptive

# Redis-backed adaptive tests
cargo test -p nemo-flow-adaptive --features redis-backend redis_tests
```

For project-level documentation, start with:

- the repo root `README.md`
- `docs/use-adaptive-optimization/configure.md`
- `docs/use-adaptive-optimization/adaptive-components.md`
- `docs/about/concepts/plugins.md`
