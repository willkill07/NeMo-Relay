<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# NeMo Flow Codex Observability

This package installs Codex hook entries that forward canonical Codex hook JSON
to `nemo-flow-sidecar` at `/hooks/codex`.

Codex CLI is fully supported for local sessions when hooks and provider routing
are configured locally. Codex GUI or app sessions are supported only when they
run on the same machine and honor the same local hook/plugin config and provider
routing. Cloud or remote Codex tasks are partial or unsupported for local
sidecar LLM capture.

## Files

- `.codex-plugin/plugin.json` describes the installable Codex plugin package.
- `hooks/hooks.json` contains hook entries that run
  `nemo-flow-sidecar hook-forward codex`.

## Start The Sidecar

Build or install the sidecar binary so `nemo-flow-sidecar` is on `PATH`.

Start a local sidecar with ATIF export enabled:

```bash
NEMO_FLOW_ATIF_DIR=.nemo-flow/atif \
nemo-flow-sidecar --bind 127.0.0.1:4040
```

Use a custom OpenAI-compatible upstream when needed:

```bash
nemo-flow-sidecar \
  --bind 127.0.0.1:4040 \
  --openai-base-url https://api.openai.com \
  --atif-dir .nemo-flow/atif
```

## Install Hooks

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

Install for Codex CLI and local GUI/app sessions:

```bash
nemo-flow-sidecar install codex \
  --scope user \
  --target both \
  --sidecar-url http://127.0.0.1:4040 \
  --atif-dir .nemo-flow/atif \
  --gateway-mode required
```

The installer merges NeMo Flow hook entries into `.codex/hooks.json`, backs up
existing config files, and enables Codex hooks in `.codex/config.toml`:

```toml
[features]
codex_hooks = true
```

## Configure LLM Gateway

For complete LLM lifecycle observability, configure the local Codex model
provider `base_url` to use the sidecar gateway:

```toml
[model_providers.openai]
base_url = "http://127.0.0.1:4040"
```

The sidecar forwards OpenAI-compatible `/v1/responses`,
`/v1/chat/completions`, and model routes without rewriting provider request or
response JSON.

Hook-only mode observes Codex sessions, prompts, subagents, tools, compaction,
and stop events. It does not observe provider request and response lifecycle
unless model traffic goes through the sidecar gateway.

## Smoke Test

Verify the sidecar endpoint directly:

```bash
printf '{"session_id":"smoke-codex","hook_event_name":"sessionStart"}' \
  | nemo-flow-sidecar hook-forward codex --sidecar-url http://127.0.0.1:4040
```

The command should print a Codex-compatible hook response. Most lifecycle events
return an empty JSON object.

Then run a small Codex prompt that starts a session and uses one simple tool.
The sidecar should receive hook requests for session and tool lifecycle events.

## Verify ATIF Export

End the Codex session and confirm that ATIF was written:

```bash
ls .nemo-flow/atif
```

The sidecar writes `<session-id>.atif.json` when it receives a session-end hook
for a session with ATIF enabled.

## Troubleshooting

If no hook events arrive, confirm `codex_hooks = true`, Codex loaded the
expected `.codex/hooks.json`, `nemo-flow-sidecar` is on `PATH`, and the sidecar
is listening on the configured URL.

If hooks arrive but LLM spans are missing, confirm the active Codex process uses
a provider `base_url` of `http://127.0.0.1:4040`. For GUI/app sessions, confirm
the session is local rather than remote.

If ATIF is missing, confirm `--atif-dir` or `NEMO_FLOW_ATIF_DIR` is configured
and that the sidecar process can write to the directory.
