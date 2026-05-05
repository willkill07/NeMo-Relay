<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Codex Sidecar Guide

Use this guide to observe local Codex CLI sessions and local Codex GUI or app
sessions that honor the same local config and gateway routing. Cloud or remote
Codex tasks are partial or unsupported for local sidecar LLM capture because the
local sidecar cannot observe provider traffic that never reaches the machine.

## Install Hooks

Inspect the generated config first:

```bash
nemo-flow-sidecar install codex \
  --scope user \
  --target both \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode required \
  --dry-run \
  --print
```

Then install it:

```bash
nemo-flow-sidecar install codex \
  --scope user \
  --target both \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode required
```

The packaged Codex plugin files live in `integrations/coding-agents/codex/`.
The installer merges hook entries into `.codex/hooks.json` and enables hooks in
`.codex/config.toml` with:

```toml
[features]
codex_hooks = true
```

## Start The Sidecar

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Use `--openai-base-url` if the sidecar should forward OpenAI-compatible traffic
to a provider other than `https://api.openai.com`.

## Configure The Gateway

For Codex CLI, configure the model provider `base_url` to use the sidecar:

```toml
[model_providers.openai]
base_url = "http://127.0.0.1:4040"
```

Local Codex GUI or app sessions have the same support level only when they read
the same local hook/plugin config and provider routing. Cloud tasks may still
emit some lifecycle hooks, but complete LLM lifecycle capture requires model
traffic to pass through the sidecar.

## Smoke Test

Run a small Codex prompt that starts a session and uses one simple tool. Then
check hook forwarding directly:

```bash
printf '{"session_id":"smoke-codex","hook_event_name":"sessionStart"}' \
  | nemo-flow-sidecar hook-forward codex --sidecar-url http://127.0.0.1:4040
```

The response should match Codex hook semantics. For most lifecycle events it is
an empty JSON object.

## Verify Export

End the Codex session and confirm ATIF exists:

```bash
ls .nemo-flow/atif
```

The sidecar writes `<session-id>.atif.json` on session end. If the file is
missing, confirm `codex_hooks = true`, hook config loading, and `--atif-dir` or
`NEMO_FLOW_ATIF_DIR`.

## Troubleshoot LLM Lifecycle

If agent/tool events exist but LLM spans are missing, the provider `base_url` is
not pointing at the sidecar for the active Codex process. If only GUI sessions
are missing spans, confirm the GUI is using local provider configuration rather
than a remote execution path.
