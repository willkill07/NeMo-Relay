<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Claude Code Sidecar Guide

Use this guide to observe Claude Code sessions with NeMo Flow. Claude Code is
the supported integration target. The Claude application, Claude web, and Claude
desktop sessions are unsupported unless they expose the same local hook and
gateway controls as Claude Code.

## Install Hooks

Inspect the generated config first:

```bash
nemo-flow-sidecar install claude-code \
  --scope user \
  --target cli \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode required \
  --dry-run \
  --print
```

Then install it:

```bash
nemo-flow-sidecar install claude-code \
  --scope user \
  --target cli \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode required
```

The packaged hook files live in
`integrations/coding-agents/claude-code/`. The installer merges equivalent hook
entries into `.claude/settings.json` and backs up an existing file before
writing.

## Start The Sidecar

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Add `NEMO_FLOW_OPENINFERENCE_ENDPOINT` or `--openinference-endpoint` when the
session should also export OpenInference traces.

## Configure The Gateway

Route Claude Code Anthropic traffic through the sidecar:

```bash
export ANTHROPIC_BASE_URL=http://127.0.0.1:4040
```

The sidecar forwards Anthropic `/v1/messages`, `/v1/messages/count_tokens`, and
model routes without rewriting provider JSON. Hook-only mode observes agent,
subagent, and tool lifecycle, but it cannot prove complete LLM lifecycle without
this gateway routing.

## Smoke Test

Run a small Claude Code prompt that starts a session and uses one simple tool.
Then check that hook forwarding reaches the sidecar:

```bash
printf '{"session_id":"smoke-claude","hook_event_name":"SessionStart"}' \
  | nemo-flow-sidecar hook-forward claude-code --sidecar-url http://127.0.0.1:4040
```

The response should be valid Claude Code hook JSON. For most lifecycle events it
is an allow/continue response.

## Verify Export

End the Claude Code session and confirm that session-end closed the NeMo Flow
agent scope and wrote ATIF:

```bash
ls .nemo-flow/atif
```

The sidecar exports `<session-id>.atif.json` on session end. If no file appears,
confirm that `SessionEnd` hooks fire, `--atif-dir` or `NEMO_FLOW_ATIF_DIR` is
set, and the sidecar process can write to the directory.

## Troubleshoot LLM Lifecycle

Missing hooks usually means Claude Code did not load the local hook config or
the `nemo-flow-sidecar` binary is not on `PATH`.

Missing LLM spans with present hook spans means Anthropic traffic is not routed
through the sidecar. Verify `ANTHROPIC_BASE_URL` in the Claude Code process
environment and confirm that requests hit `/v1/messages`.
