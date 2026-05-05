<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# NeMo Flow Coding-Agent Observability Integrations

This directory contains installable hook integrations for coding agents that
should be observed by `nemo-flow-sidecar`.

The sidecar combines two observability paths:

- Agent lifecycle hooks for sessions, prompts, subagents, tool calls,
  compaction, responses, and stop events.
- A passthrough LLM gateway for OpenAI-compatible and Anthropic-compatible
  provider traffic.

Hook integrations preserve each coding agent's canonical hook payload. They do
not wrap the payload in a shared NeMo Flow envelope. Sidecar-specific settings
travel through hook command arguments and HTTP headers.

## Packages

- `claude-code/` installs Claude Code hook entries targeting
  `POST /hooks/claude-code`.
- `codex/` installs Codex hook entries targeting `POST /hooks/codex` and enables
  `codex_hooks = true`.
- `cursor/` installs a Cursor `.cursor/hooks.json` bundle targeting
  `POST /hooks/cursor`.

## Common Setup

Build or install the sidecar binary so `nemo-flow-sidecar` is on `PATH`.

Start the sidecar:

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Install an integration:

```bash
nemo-flow-sidecar install claude-code --scope user --target cli --sidecar-url http://127.0.0.1:4040
nemo-flow-sidecar install codex --scope user --target both --sidecar-url http://127.0.0.1:4040
nemo-flow-sidecar install cursor --scope project --target gui --sidecar-url http://127.0.0.1:4040
```

Inspect generated changes before writing:

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

The installer backs up existing config files, merges only NeMo Flow hook
entries, and avoids adding duplicate NeMo Flow entries on repeated runs.

## Common Options

The installer writes hook commands that call:

```bash
nemo-flow-sidecar hook-forward <agent>
```

`hook-forward` reads the canonical hook JSON from standard input, forwards it to
the matching sidecar endpoint, and prints the vendor-specific hook response.

Useful install options:

- `--atif-dir <path>` writes ATIF trajectories on session end.
- `--openinference-endpoint <url>` exports OpenInference traces.
- `--profile <name>` records a sidecar profile name in session metadata.
- `--session-metadata '<json>'` adds structured metadata to the agent begin
  event.
- `--plugin-config '<json>'` records scope-local plugin configuration metadata.
- `--gateway-mode hook-only|passthrough|required` records the intended gateway
  mode for the session.
- `--fail-closed` can be added to generated hook commands when the agent should
  block on hook delivery failures. The default is fail-open.

## LLM Gateway

Complete LLM lifecycle observability requires model traffic to pass through the
sidecar. Hook-only mode observes agent, subagent, and tool lifecycle, but it
cannot observe provider request and response lifecycle when the coding agent
sends model traffic directly to an upstream provider or remote service.

The sidecar exposes these passthrough routes:

- `POST /v1/responses`
- `POST /v1/chat/completions`
- `POST /v1/messages`
- `POST /v1/messages/count_tokens`
- `GET /v1/models`

Configure each coding agent's provider base URL to use
`http://127.0.0.1:4040` where that agent supports local provider routing.

## Verify Export

Run a coding-agent session that starts, uses one tool, and ends. Then confirm
that ATIF was written:

```bash
ls .nemo-flow/atif
```

The sidecar writes `<session-id>.atif.json` when it receives a session-end hook
for a session with ATIF configured.
