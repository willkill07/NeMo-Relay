<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Advanced Guide: Coding-Agent Gateway Sidecar

The `nemo-flow-sidecar` binary observes coding agents that do not expose every
LLM call site directly. It combines agent-specific hook endpoints with a
passthrough LLM gateway so NeMo Flow owns both the agent lifecycle and the model
request lifecycle.

Use the sidecar when you need one observability boundary for OpenAI Codex,
Claude Code, and Cursor without replacing each agent's canonical hook payload.

## Hook Endpoints

Each hook endpoint accepts the agent's native hook JSON directly. Do not wrap
the payload in a shared sidecar envelope.

- `POST /hooks/codex` accepts Codex hook JSON and returns the Codex-compatible
  hook response object.
- `POST /hooks/claude-code` accepts Claude Code hook JSON and returns
  Claude-compatible fields such as `continue` and permission decisions when the
  hook event supports them.
- `POST /hooks/cursor` accepts Cursor hook JSON and returns Cursor-compatible
  fields such as `continue`, `permission`, `user_message`, and `agent_message`
  when the hook event supports them.

The adapters preserve vendor fields such as session IDs, working directories,
transcript paths, model names, tool payloads, shell payloads, MCP payloads, file
payloads, user identity, and subagent metadata in NeMo Flow event metadata.

## Gateway Routes

Route all coding-agent LLM traffic through the sidecar when full LLM lifecycle
observability is required.

- `POST /v1/responses`
- `POST /v1/chat/completions`
- `POST /v1/messages`
- `POST /v1/messages/count_tokens`
- `GET /v1/models`

The gateway forwards raw provider JSON without rewriting OpenAI or Anthropic
payload schemas. It removes only hop-by-hop transport headers, forwards
streaming responses as streams, and emits NeMo Flow LLM start and end events
under the active session scope.

## Session Configuration

Sidecar-specific configuration travels through hook registration settings,
headers, environment variables, or a referenced sidecar profile. It must not
replace the coding agent's canonical hook schema.

Common headers are:

- `x-nemo-flow-session-id`
- `x-nemo-flow-config-profile`
- `x-nemo-flow-session-metadata`
- `x-nemo-flow-plugin-config`
- `x-nemo-flow-openinference-endpoint`
- `x-nemo-flow-atif-dir`

Common environment variables are:

- `NEMO_FLOW_SIDECAR_BIND`
- `NEMO_FLOW_OPENAI_BASE_URL`
- `NEMO_FLOW_ANTHROPIC_BASE_URL`
- `NEMO_FLOW_OPENINFERENCE_ENDPOINT`
- `NEMO_FLOW_ATIF_DIR`

Per-session configuration controls the scope-local OpenInference subscriber,
the ATIF exporter, structured metadata on the top-level agent begin event, and
the plugin configuration metadata associated with the session.

## Runtime Mapping

The sidecar normalizes vendor hook payloads into private internal events before
calling NeMo Flow APIs.

- Agent start opens a top-level `ScopeType::Agent` scope on a dedicated
  `ScopeStackHandle`.
- Subagent start opens a child `ScopeType::Agent` scope. Subagent stop closes
  that scope when it is still active.
- Tool pre-use starts a NeMo Flow tool span. Tool post-use, denial, or failure
  closes it.
- Prompt, response, compaction, notification, and unknown hook events become
  mark events under the active session scope.
- Gateway requests emit NeMo Flow LLM start and end events under the active
  session scope.

Cursor hook-only mode observes agent, subagent, and tool lifecycle. To observe
Cursor LLM lifecycle completely, configure Cursor model traffic to use the
sidecar gateway.

## Install Integrations

The repository includes installable integration packages under
`integrations/coding-agents/` and an installer in the sidecar binary.

```bash
nemo-flow-sidecar install claude-code --scope user --target cli --sidecar-url http://127.0.0.1:4040
nemo-flow-sidecar install codex --scope user --target both --sidecar-url http://127.0.0.1:4040
nemo-flow-sidecar install cursor --scope project --target gui --sidecar-url http://127.0.0.1:4040
```

Use `--dry-run` to see which files would be changed. Use `--print` to print the
merged file contents. Existing config files are backed up before the installer
writes replacement files, and generated hook entries are appended only when the
same NeMo Flow entry is not already present.

Common install options become hook-forwarding command arguments and sidecar
headers:

- `--atif-dir` sets `x-nemo-flow-atif-dir`.
- `--openinference-endpoint` sets `x-nemo-flow-openinference-endpoint`.
- `--profile` sets `x-nemo-flow-config-profile`.
- `--session-metadata` sets `x-nemo-flow-session-metadata`.
- `--plugin-config` sets `x-nemo-flow-plugin-config`.
- `--gateway-mode hook-only|passthrough|required` sets
  `x-nemo-flow-gateway-mode`.

The generated hooks run:

```bash
nemo-flow-sidecar hook-forward <agent>
```

`hook-forward` reads the canonical hook payload from standard input, sends it to
the matching endpoint, and prints the endpoint response. It fails open by
default so observability outages do not block the coding agent. Add
`--fail-closed` only when policy requires hook delivery to block the agent.

## Agent Guides

Use the per-agent guide for end-to-end setup, smoke tests, and GUI or
application-mode caveats.

- [Claude Code Sidecar Guide](coding-agent-claude-code.md)
- [Codex Sidecar Guide](coding-agent-codex.md)
- [Cursor Sidecar Guide](coding-agent-cursor.md)

Each guide covers plugin or hook installation, sidecar startup, gateway routing,
hook smoke tests, ATIF export verification on session end, and troubleshooting
missing LLM lifecycle data.
