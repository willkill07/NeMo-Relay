<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Advanced Guide: Tune Adaptive Behavior

Use this guide after the adaptive component is initialized and observing representative NeMo Flow traffic.

## What You Tune

Adaptive optimization has several independent control surfaces:

- State backend and retention.
- Telemetry subscribers and learners.
- Adaptive hint injection.
- Tool parallelism mode.
- Adaptive cache governor behavior.
- Policy and rollout controls.

Tune one area at a time. Keep a stable baseline so you can tell whether a change improves latency, cost, cache behavior, or reliability.

## Before You Start

Complete [Basic Guide: Configure Adaptive Optimization](configure.md). You should have:

- Instrumented tool or LLM calls.
- Adaptive telemetry enabled.
- A stable `agent_id`.
- At least one representative workflow that you can run repeatedly.
- A way to inspect lifecycle events, adaptive reports, or exported traces.

## Tuning Workflow

Follow this workflow to tune adaptive behavior from measured baselines rather than
guesses.

1. Record baseline latency, tool count, model usage, cache behavior, and error rate.
2. Enable one adaptive area in observe-only or low-impact mode.
3. Run the same representative workflow several times.
4. Compare events, reports, and application behavior against the baseline.
5. Promote the setting only when the measured behavior is stable.
6. Document the setting and the workflow it was tuned for.

## State Tuning

Use in-memory state for local development, tests, and short-lived experiments. Use Redis-backed state when multiple workers need shared observations or when adaptive behavior should survive process restarts.

Keep these decisions explicit:

- Which logical agent owns the state.
- How long observations remain useful.
- Whether state can be shared across tenants.
- How state is reset during tests or rollbacks.

## Telemetry Tuning

Telemetry controls what adaptive learners observe. Start with only the learners needed for the feature you are testing.

Use telemetry to answer concrete questions:

- Which tools are frequently independent and safe to parallelize?
- Which prompt sections are stable across repeated runs?
- Which requests carry enough metadata for hints or cache planning?
- Which failures correlate with specific runtime paths?

If telemetry volume is high, reduce the enabled learners before reducing instrumentation. Instrumentation is also used by subscribers and exporters outside adaptive optimization.

## Adaptive Hint Tuning

Adaptive hints attach guidance to model requests. Use them when downstream code or provider adapters can safely consume hint metadata.

Tune hints conservatively:

1. Start with a low priority so existing request intercepts run first.
2. Inject hints into a predictable header or body path.
3. Validate the transformed request before it reaches the provider.
4. Disable `break_chain` unless the adaptive hint should be the final request transform.

## Tool Parallelism Tuning

Tool parallelism should move through three phases:

| Mode | Use When | Expected Behavior |
|---|---|---|
| `observe_only` | You are gathering data | Execution does not change |
| `inject_hints` | Downstream code can interpret guidance | Runtime adds guidance but does not own scheduling |
| `schedule` | You want adaptive behavior to influence execution strategy | Runtime can affect scheduling decisions |

Only use `schedule` when tool callbacks are idempotent or safe to run under the planned concurrency model.

## Adaptive Cache Governor Tuning

Enable the adaptive cache governor when repeated LLM requests contain stable prompt sections that can benefit from provider prompt caching.

Tune these fields together:

- `provider`: Set to `anthropic`, `openai`, or `passthrough` based on the provider surface.
- `observation_window`: Increase when prompts vary across runs and stability needs more samples.
- `stability_thresholds`: Raise thresholds when cache breakpoints are too aggressive.
- Policy controls: Keep reporting enabled while tuning so decisions are auditable.

Use `passthrough` when you want to keep observing prompt structure without applying provider-specific cache planning.

## Diagnostics and Runtime Variables

NeMo Flow does not require application-level environment variables for normal adaptive runtime use. Prefer explicit adaptive config objects for application behavior.

Use these variables only for adjacent diagnostics and tests:

| Variable | Scope | Purpose |
|---|---|---|
| `NEMO_FLOW_ACG_DEBUG` | Adaptive cache-governor diagnostics | Enables cache-governor debug diagnostics in adaptive internals. |
| `NEMO_FLOW_RUN_REDIS_TESTS` | Test workflows | Enables Redis-backed adaptive tests. |

Internal variables such as `NEMO_FLOW_BINDING_KIND` and `NEMO_FLOW_RUNTIME_OWNER` are binding and test controls. Do not set them in application code unless a maintainer asks you to debug runtime ownership behavior.

## Validation Checklist

For each change, verify:

- Application results are unchanged unless the change intentionally affects scheduling.
- Emitted events still include the expected scope, tool, and LLM spans.
- Adaptive reports explain the decision that changed behavior.
- Latency, token usage, or cache behavior improves on representative traffic.
- Rollback is a configuration change, not a code change.

## Common Issues

Check these symptoms first when the workflow does not behave as expected.

- **Observed behavior is noisy**: Increase the observation window or reduce the workflow variance during tuning.
- **Hints conflict with application intercepts**: Adjust priority or disable `break_chain`.
- **Parallelism creates race conditions**: Return to `observe_only` and audit tool idempotency.
- **Cache planning is unstable**: Use more samples, raise stability thresholds, or set provider to `passthrough`.
- **State leaks across tenants**: Scope state by `agent_id` and deployment boundaries.

## Next Steps

Use these links to continue from this workflow into the next related task.

- Review [Code Examples](code-examples.md) for binding APIs, defaults, and ACG threshold overrides.
- Review [Advanced Guide: Configure Adaptive Components](adaptive-components.md) for plugin-level adaptive configuration.
- Export traces with [Advanced Guide: Export OpenInference Data](../export-observability-data/advanced-guide.md) to compare behavior across runs.
