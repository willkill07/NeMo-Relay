<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# NeMo Flow Claude Code Observability

This package installs Claude Code hook entries that forward canonical Claude Code
hook JSON to `nemo-flow-sidecar` at `/hooks/claude-code`.

Claude Code is the supported Claude integration target. Claude application,
Claude web, and Claude desktop sessions are unsupported unless they expose the
same local hook and gateway controls as Claude Code.

## Files

- `.claude-plugin/plugin.json` describes the installable Claude Code hook
  package.
- `hooks/hooks.json` contains hook entries that run
  `nemo-flow-sidecar hook-forward claude-code`.

## Start The Sidecar

Build or install the sidecar binary so `nemo-flow-sidecar` is on `PATH`.

Start a local sidecar with ATIF export enabled:

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Add OpenInference export when needed:

```bash
nemo-flow-sidecar \
  --bind 127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --openinference-endpoint http://127.0.0.1:4318/v1/traces
```

## Install Hooks

Inspect the generated config before writing:

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

Install for Claude Code:

```bash
nemo-flow-sidecar install claude-code \
  --scope user \
  --target cli \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode required
```

The installer merges NeMo Flow hook entries into `.claude/settings.json` and
backs up any existing file before writing. Sidecar-specific options are stored
in the generated hook command and forwarded as HTTP headers.

## Configure LLM Gateway

For complete LLM lifecycle observability, route Claude Code's Anthropic traffic
through the sidecar:

```bash
export ANTHROPIC_BASE_URL=http://127.0.0.1:4040
```

The sidecar forwards Anthropic `/v1/messages`, `/v1/messages/count_tokens`, and
model routes without rewriting provider request or response JSON.

Hook-only mode observes Claude Code sessions, prompts, subagents, tools,
compaction, and stop events. It does not observe provider request and response
lifecycle unless model traffic goes through the sidecar gateway.

## Smoke Test

Verify the sidecar endpoint directly:

```bash
printf '{"session_id":"smoke-claude","hook_event_name":"SessionStart"}' \
  | nemo-flow-sidecar hook-forward claude-code --sidecar-url http://127.0.0.1:4040
```

The command should print a Claude-compatible continue response.

Then run a small Claude Code prompt that starts a session and uses one simple
tool. The sidecar should receive hook requests for session and tool lifecycle
events.

## Verify ATIF Export

End the Claude Code session and confirm that ATIF was written:

```bash
ls .nemo-flow/atif
```

The sidecar writes `<session-id>.atif.json` when it receives `SessionEnd` for a
session with ATIF enabled.

## Troubleshooting

If no hook events arrive, confirm `nemo-flow-sidecar` is on `PATH`, Claude Code
loaded `.claude/settings.json`, and the sidecar is listening on the configured
URL.

If hooks arrive but LLM spans are missing, confirm `ANTHROPIC_BASE_URL` is set
in the Claude Code process environment and points to `http://127.0.0.1:4040`.

If ATIF is missing, confirm `--atif-dir` or `NEMO_FLOW_ATIF_DIR` is configured
and that the sidecar process can write to the directory.
