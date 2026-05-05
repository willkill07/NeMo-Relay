<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Cursor Sidecar Guide

Use this guide to observe Cursor hook lifecycle events with NeMo Flow. The
repository ships a Cursor hook bundle under `integrations/coding-agents/cursor/`
because this integration does not assume an official Cursor plugin package
format.

Cursor GUI or IDE sessions can provide agent, subagent, tool, shell, MCP, file,
and response lifecycle events through `.cursor/hooks.json`. Complete LLM
lifecycle observability additionally requires Cursor model traffic to route
through the sidecar gateway if your Cursor build exposes that configuration.

Cursor CLI support must be verified separately with `cursor-agent`. If CLI hooks
do not fire, treat Cursor CLI support as hook-limited and gateway-only where
model routing is configurable.

## Install Hooks

Inspect the generated config first:

```bash
nemo-flow-sidecar install cursor \
  --scope project \
  --target gui \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode passthrough \
  --dry-run \
  --print
```

Then install it:

```bash
nemo-flow-sidecar install cursor \
  --scope project \
  --target gui \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode passthrough
```

The installer merges NeMo Flow entries into `.cursor/hooks.json` and backs up an
existing file before writing.

## Start The Sidecar

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Use `--openai-base-url` or `--anthropic-base-url` when the sidecar should
forward to non-default upstream providers.

## Configure The Gateway

If Cursor exposes provider base URL configuration, point OpenAI-compatible or
Anthropic-compatible traffic at:

```text
http://127.0.0.1:4040
```

Hook-only Cursor mode observes agent and tool lifecycle but cannot provide
complete LLM lifecycle. Missing LLM spans are expected when Cursor sends model
traffic directly to the provider or through a remote service.

## Smoke Test

Run a small Cursor GUI session that starts an agent and uses one simple tool.
Then check hook forwarding directly:

```bash
printf '{"session_id":"smoke-cursor","hook_event_name":"sessionStart"}' \
  | nemo-flow-sidecar hook-forward cursor --sidecar-url http://127.0.0.1:4040
```

For Cursor CLI, run an equivalent `cursor-agent` session and verify the sidecar
receives hook requests. If no hook requests arrive, document that CLI version as
hook-limited and rely only on gateway observability where provider routing is
available.

## Verify Export

End the Cursor session and confirm ATIF exists:

```bash
ls .nemo-flow/atif
```

The sidecar writes `<session-id>.atif.json` on session end. If the file is
missing, confirm Cursor loaded `.cursor/hooks.json`, the sidecar binary is on
`PATH`, and `--atif-dir` or `NEMO_FLOW_ATIF_DIR` is configured.

## Troubleshoot LLM Lifecycle

If Cursor hook events appear but LLM spans are missing, provider traffic is not
routed through the sidecar. Confirm the active Cursor GUI or CLI mode supports
provider base URL configuration for the model path being used.
