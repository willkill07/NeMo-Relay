---
name: nemo-flow-configure-optimizer
description: Configure the NeMo Flow adaptive layer and plugins through the shared plugin system; use this when users still say optimizer
author: NVIDIA Corporation and Affiliates
license: Apache-2.0
---


# Configure Adaptive Layer

Use this skill when an application already intends to use adaptive features
(sometimes still called the optimizer) and now needs a correct configuration.

There is no separate public adaptive runtime object. Adaptive is configured as a
top-level plugin component inside the shared plugin system.

## Embedded Configuration Model

- The top-level adaptive object contains `version`, `agent_id`, `state`,
  `telemetry`, `adaptive_hints`, `tool_parallelism`, `acg`, and `policy`.
- Wrap the adaptive object in an adaptive `ComponentSpec`, insert it into the
  shared plugin config `components` list, validate the plugin config, then
  initialize the plugin system.
- Python uses `nemo_flow.adaptive.AdaptiveConfig(...)`,
  `nemo_flow.adaptive.ComponentSpec(...)`, and
  `nemo_flow.plugin.PluginConfig(...)`.
- Node.js uses `require("nemo-flow-node/adaptive")` helpers such as
  `defaultConfig()`, `inMemoryBackend()`, `toolParallelismConfig(...)`, and
  `ComponentSpec(...)`, then activates through `nemo-flow-node/plugin`.
- Rust uses `nemo_flow_adaptive::{AdaptiveConfig, ComponentSpec, ...}` and
  `nemo_flow::plugin::{validate_plugin_config, initialize_plugins}`.
- Go uses `adaptive.NewConfig()`, `adaptive.NewInMemoryBackend()`,
  `adaptive.NewComponentSpec(...)`, and the top-level plugin wrappers.
- Start with an in-memory state backend for local development and process-local
  experiments. Use Redis-backed state only when state must survive restarts or
  be shared across workers.
- Enable telemetry when adaptive components need to observe runtime events.
- Keep adaptive hints disabled or low-priority until downstream model calls can
  safely receive hint metadata.
- Keep tool parallelism in `observe_only` until representative traffic proves
  scheduling is safe.
- Keep the adaptive cache governor disabled initially; enable it later for
  stable prompt-cache planning.
- The plugin config uses `version`, `components`, and `policy`. Policy controls
  how unknown components, unknown fields, and unsupported values are handled.
- Plugins install runtime behavior such as subscribers, guardrails, intercepts,
  and related helpers. Adaptive is a built-in plugin component, not a separate
  runtime model.

## Default Path

1. Build the shared plugin config document or binding-native helper config.
2. Add one top-level `adaptive.ComponentSpec(...)`.
3. Choose the state backend.
4. Enable only the adaptive sections you need: `telemetry`,
   `adaptive_hints`, and `tool_parallelism`.
5. Validate the config.
6. Initialize through the shared plugin system.
7. Clear or replace the plugin configuration cleanly when the app lifecycle
   changes.

## Defaults To Remember

- `adaptive_hints.priority` defaults to `100`, `break_chain` to `false`,
  `inject_header` to `true`, and `inject_body_path` to `nvext.agent_hints`.
- `tool_parallelism.mode` defaults to `observe_only`.
- `acg.provider` defaults to `passthrough`, with priority `50` and observation
  window `100`.

## Checklist

- [ ] Adaptive is modeled as a top-level plugin component, not a nested runtime
- [ ] Backend chosen (`in_memory` first unless persistence is required)
- [ ] Adaptive sections chosen explicitly
- [ ] Config validated before initialization
- [ ] Custom plugins added as sibling top-level components when used
- [ ] Plugin lifecycle matched to the app lifecycle

## Related Skills

- `nemo-flow-start-optimizer`
- `nemo-flow-use-optimizer-hints`
- `nemo-flow-debug-runtime-integration`
