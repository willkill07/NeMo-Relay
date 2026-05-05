<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# NeMo Flow Cursor Observability

This package is a Cursor hook bundle, not an official Cursor plugin package. It
installs `.cursor/hooks.json` entries that forward canonical Cursor hook JSON to
`nemo-flow-sidecar` at `/hooks/cursor`.

Cursor GUI or IDE sessions can provide agent, subagent, tool, shell, MCP, file,
and response lifecycle events through `.cursor/hooks.json`. Complete LLM
lifecycle observability additionally requires Cursor model traffic to route
through the sidecar gateway if the active Cursor build exposes provider base URL
configuration.

Cursor CLI support must be verified separately with `cursor-agent`. If CLI hooks
do not fire, treat Cursor CLI support as hook-limited and gateway-only where
model routing is configurable.

## Files

- `.cursor/hooks.json` contains hook entries that run
  `nemo-flow-sidecar hook-forward cursor`.

## Start The Sidecar

Build or install the sidecar binary so `nemo-flow-sidecar` is on `PATH`.

Start a local sidecar with ATIF export enabled:

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Use custom upstreams when needed:

```bash
nemo-flow-sidecar \
  --bind 127.0.0.1:4040 \
  --openai-base-url https://api.openai.com \
  --anthropic-base-url https://api.anthropic.com \
  --atif-dir .nemo-flow/atif
```

## Install Hooks

Inspect generated changes before writing:

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

Install for a project-local Cursor GUI or IDE session:

```bash
nemo-flow-sidecar install cursor \
  --scope project \
  --target gui \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode passthrough
```

The installer merges NeMo Flow hook entries into `.cursor/hooks.json` and backs
up an existing file before writing.

## Configure LLM Gateway

If Cursor exposes provider base URL configuration for the model path being used,
point OpenAI-compatible or Anthropic-compatible traffic at:

```text
http://127.0.0.1:4040
```

The sidecar forwards OpenAI-compatible `/v1/responses`,
`/v1/chat/completions`, Anthropic-compatible `/v1/messages`, token-count, and
model routes without rewriting provider request or response JSON.

Hook-only mode observes Cursor agent and tool lifecycle. Missing LLM spans are
expected when Cursor sends model traffic directly to the provider or through a
remote service.

## Smoke Test

Verify the sidecar endpoint directly:

```bash
printf '{"session_id":"smoke-cursor","hook_event_name":"sessionStart"}' \
  | nemo-flow-sidecar hook-forward cursor --sidecar-url http://127.0.0.1:4040
```

The command should print a Cursor-compatible continue response.

Then run a small Cursor GUI session that starts an agent and uses one simple
tool. The sidecar should receive hook requests for session and tool lifecycle
events.

For Cursor CLI, run an equivalent `cursor-agent` session and verify that the
sidecar receives hook requests. If no hook requests arrive, treat that CLI
version as hook-limited.

## Verify ATIF Export

End the Cursor session and confirm that ATIF was written:

```bash
ls .nemo-flow/atif
```

The sidecar writes `<session-id>.atif.json` when it receives a session-end hook
for a session with ATIF enabled.

## Troubleshooting

If no hook events arrive, confirm Cursor loaded `.cursor/hooks.json`,
`nemo-flow-sidecar` is on `PATH`, and the sidecar is listening on the configured
URL.

If hooks arrive but LLM spans are missing, confirm the active Cursor GUI or CLI
mode supports provider base URL configuration and points provider traffic to
`http://127.0.0.1:4040`.

If ATIF is missing, confirm `--atif-dir` or `NEMO_FLOW_ATIF_DIR` is configured
and that the sidecar process can write to the directory.
