// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod claude_code;
pub(crate) mod codex;
pub(crate) mod cursor;

use axum::http::HeaderMap;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::config::header_string;
use crate::model::{AgentKind, NormalizedEvent, SessionEvent, SubagentEvent, ToolEvent};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AdapterOutcome {
    pub(crate) events: Vec<NormalizedEvent>,
    pub(crate) response: Value,
}

pub(super) struct ClassificationRules<'a> {
    kind: AgentKind,
    agent_start: &'a [&'a str],
    agent_end: &'a [&'a str],
    subagent_start: &'a [&'a str],
    subagent_end: &'a [&'a str],
    tool_start: &'a [&'a str],
    tool_end: &'a [&'a str],
}

fn session_id(payload: &Value, headers: &HeaderMap) -> String {
    header_string(headers, "x-nemo-flow-session-id")
        .or_else(|| header_string(headers, "x-claude-code-session-id"))
        .or_else(|| string_at(payload, &["session_id"]))
        .or_else(|| string_at(payload, &["sessionId"]))
        .or_else(|| string_at(payload, &["session", "id"]))
        .or_else(|| string_at(payload, &["conversation_id"]))
        .or_else(|| string_at(payload, &["conversationId"]))
        .unwrap_or_else(|| format!("hook-{}", Uuid::now_v7()))
}

fn event_name(payload: &Value) -> String {
    string_at(payload, &["hook_event_name"])
        .or_else(|| string_at(payload, &["event_name"]))
        .or_else(|| string_at(payload, &["eventName"]))
        .or_else(|| string_at(payload, &["event"]))
        .or_else(|| string_at(payload, &["type"]))
        .or_else(|| string_at(payload, &["name"]))
        .unwrap_or_else(|| "unknown".to_string())
}

fn metadata(payload: &Value, headers: &HeaderMap, kind: AgentKind, event_name: &str) -> Value {
    let mut object = Map::new();
    object.insert("agent_kind".into(), json!(kind.as_str()));
    object.insert("hook_event_name".into(), json!(event_name));
    if let Some(profile) = header_string(headers, "x-nemo-flow-config-profile") {
        object.insert("sidecar_config_profile".into(), json!(profile));
    }
    for (key, value) in [
        ("cwd", string_at(payload, &["cwd"])),
        ("transcript_path", string_at(payload, &["transcript_path"])),
        ("project_dir", string_at(payload, &["project_dir"])),
        ("user_email", string_at(payload, &["user_email"])),
        ("model", string_at(payload, &["model"])),
    ] {
        if let Some(value) = value {
            object.insert(key.into(), json!(value));
        }
    }
    Value::Object(object)
}

fn common_session_event(payload: &Value, headers: &HeaderMap, kind: AgentKind) -> SessionEvent {
    let event_name = event_name(payload);
    SessionEvent {
        session_id: session_id(payload, headers),
        agent_kind: kind,
        event_name: event_name.clone(),
        payload: payload.clone(),
        metadata: metadata(payload, headers, kind, &event_name),
    }
}

fn common_subagent_event(payload: &Value, headers: &HeaderMap, kind: AgentKind) -> SubagentEvent {
    let session = common_session_event(payload, headers, kind);
    let subagent_id = subagent_id(payload)
        .or_else(|| header_string(headers, "x-nemo-flow-subagent-id"))
        .unwrap_or_else(|| "subagent".to_string());
    SubagentEvent {
        session_id: session.session_id,
        agent_kind: kind,
        event_name: session.event_name,
        subagent_id,
        payload: session.payload,
        metadata: session.metadata,
    }
}

fn common_tool_event(payload: &Value, headers: &HeaderMap, kind: AgentKind) -> ToolEvent {
    let session = common_session_event(payload, headers, kind);
    let tool_call_id = string_at(payload, &["tool_call_id"])
        .or_else(|| string_at(payload, &["toolCallId"]))
        .or_else(|| string_at(payload, &["call_id"]))
        .or_else(|| string_at(payload, &["tool", "id"]))
        .or_else(|| string_at(payload, &["tool_input", "id"]))
        .or_else(|| string_at(payload, &["id"]))
        .unwrap_or_else(|| format!("tool-{}", Uuid::now_v7()));
    let tool_name = string_at(payload, &["tool_name"])
        .or_else(|| string_at(payload, &["toolName"]))
        .or_else(|| string_at(payload, &["tool", "name"]))
        .or_else(|| string_at(payload, &["tool_input", "name"]))
        .or_else(|| string_at(payload, &["name"]))
        .unwrap_or_else(|| "unknown_tool".to_string());
    let arguments = value_at(payload, &["tool_input"])
        .or_else(|| value_at(payload, &["input"]))
        .or_else(|| value_at(payload, &["arguments"]))
        .or_else(|| value_at(payload, &["args"]))
        .unwrap_or(Value::Null);
    let result = value_at(payload, &["tool_output"])
        .or_else(|| value_at(payload, &["output"]))
        .or_else(|| value_at(payload, &["result"]))
        .unwrap_or(Value::Null);
    ToolEvent {
        session_id: session.session_id,
        agent_kind: kind,
        event_name: session.event_name,
        tool_call_id,
        tool_name,
        subagent_id: subagent_id(payload)
            .or_else(|| header_string(headers, "x-nemo-flow-subagent-id")),
        arguments,
        result,
        status: string_at(payload, &["status"])
            .or_else(|| string_at(payload, &["decision"]))
            .or_else(|| string_at(payload, &["permission"])),
        payload: session.payload,
        metadata: session.metadata,
    }
}

fn subagent_id(payload: &Value) -> Option<String> {
    string_at(payload, &["subagent_id"])
        .or_else(|| string_at(payload, &["subagentId"]))
        .or_else(|| string_at(payload, &["subagent", "id"]))
        .or_else(|| string_at(payload, &["agent", "id"]))
}

fn string_at(payload: &Value, path: &[&str]) -> Option<String> {
    value_at(payload, path).and_then(|value| match value {
        Value::String(value) => Some(value),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn value_at(payload: &Value, path: &[&str]) -> Option<Value> {
    let mut current = payload;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current.clone())
}

fn classify(
    payload: &Value,
    headers: &HeaderMap,
    rules: &ClassificationRules<'_>,
) -> NormalizedEvent {
    let event = event_name(payload);
    let normalized = normalize_name(&event);
    if rules
        .agent_start
        .iter()
        .any(|name| normalize_name(name) == normalized)
    {
        NormalizedEvent::AgentStarted(common_session_event(payload, headers, rules.kind))
    } else if rules
        .agent_end
        .iter()
        .any(|name| normalize_name(name) == normalized)
    {
        NormalizedEvent::AgentEnded(common_session_event(payload, headers, rules.kind))
    } else if rules
        .subagent_start
        .iter()
        .any(|name| normalize_name(name) == normalized)
    {
        NormalizedEvent::SubagentStarted(common_subagent_event(payload, headers, rules.kind))
    } else if rules
        .subagent_end
        .iter()
        .any(|name| normalize_name(name) == normalized)
    {
        NormalizedEvent::SubagentEnded(common_subagent_event(payload, headers, rules.kind))
    } else if rules
        .tool_start
        .iter()
        .any(|name| normalize_name(name) == normalized)
    {
        NormalizedEvent::ToolStarted(common_tool_event(payload, headers, rules.kind))
    } else if rules
        .tool_end
        .iter()
        .any(|name| normalize_name(name) == normalized)
    {
        NormalizedEvent::ToolEnded(common_tool_event(payload, headers, rules.kind))
    } else {
        match normalized.as_str() {
            "beforesubmitprompt" | "promptsubmitted" | "userpromptsubmit" => {
                NormalizedEvent::PromptSubmitted(common_session_event(payload, headers, rules.kind))
            }
            "afteragentresponse" | "agentresponse" | "assistantresponse" => {
                NormalizedEvent::AgentResponse(common_session_event(payload, headers, rules.kind))
            }
            "precompact" | "compaction" => {
                NormalizedEvent::Compaction(common_session_event(payload, headers, rules.kind))
            }
            "notification" => {
                NormalizedEvent::Notification(common_session_event(payload, headers, rules.kind))
            }
            _ => NormalizedEvent::HookMark(common_session_event(payload, headers, rules.kind)),
        }
    }
}

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;
    use serde_json::json;

    use super::*;
    use crate::adapters::{claude_code, codex, cursor};

    #[test]
    fn maps_claude_canonical_tool_payload() {
        let headers = HeaderMap::new();
        let outcome = claude_code::adapt(
            json!({
                "session_id": "claude-session",
                "transcript_path": "/tmp/transcript.jsonl",
                "cwd": "/workspace",
                "hook_event_name": "PreToolUse",
                "tool_name": "Read",
                "tool_input": { "file_path": "README.md" }
            }),
            &headers,
        );
        match &outcome.events[0] {
            NormalizedEvent::ToolStarted(event) => {
                assert_eq!(event.session_id, "claude-session");
                assert_eq!(event.tool_name, "Read");
                assert_eq!(event.arguments, json!({ "file_path": "README.md" }));
                assert_eq!(
                    event.metadata["transcript_path"],
                    json!("/tmp/transcript.jsonl")
                );
            }
            event => panic!("unexpected event: {event:?}"),
        }
        assert_eq!(outcome.response["continue"], json!(true));
        assert_eq!(outcome.response["permissionDecision"], json!("allow"));
    }

    #[test]
    fn maps_cursor_subagent_and_permission_response() {
        let headers = HeaderMap::new();
        let outcome = cursor::adapt(
            json!({
                "session_id": "cursor-session",
                "project_dir": "/repo",
                "user_email": "dev@example.com",
                "hook_event_name": "beforeShellExecution",
                "subagent": { "id": "worker" },
                "tool_call_id": "shell-1",
                "tool_name": "shell",
                "input": { "command": "cargo test" }
            }),
            &headers,
        );
        match &outcome.events[0] {
            NormalizedEvent::ToolStarted(event) => {
                assert_eq!(event.session_id, "cursor-session");
                assert_eq!(event.subagent_id.as_deref(), Some("worker"));
                assert_eq!(event.metadata["project_dir"], json!("/repo"));
                assert_eq!(event.metadata["user_email"], json!("dev@example.com"));
            }
            event => panic!("unexpected event: {event:?}"),
        }
        assert_eq!(outcome.response["permission"], json!("allow"));
    }

    #[test]
    fn keeps_codex_response_unwrapped() {
        let headers = HeaderMap::new();
        let outcome = codex::adapt(
            json!({
                "session_id": "codex-session",
                "hook_event_name": "sessionStart"
            }),
            &headers,
        );
        assert!(matches!(
            outcome.events[0],
            NormalizedEvent::AgentStarted(_)
        ));
        assert_eq!(outcome.response, json!({}));
    }

    #[test]
    fn normalizes_mark_style_events_and_header_session_ids() {
        let mut headers = HeaderMap::new();
        headers.insert("x-nemo-flow-session-id", "header-session".parse().unwrap());
        headers.insert("x-nemo-flow-config-profile", "coverage".parse().unwrap());

        for (event_name, expected) in [
            ("UserPromptSubmit", "prompt"),
            ("afterAgentResponse", "response"),
            ("PreCompact", "compact"),
            ("Notification", "notification"),
            ("Unrecognized.Event", "hook"),
        ] {
            let outcome = cursor::adapt(
                json!({
                    "eventName": event_name,
                    "model": "model-a",
                    "cwd": "/repo"
                }),
                &headers,
            );
            let session = match &outcome.events[0] {
                NormalizedEvent::PromptSubmitted(event) if expected == "prompt" => event,
                NormalizedEvent::AgentResponse(event) if expected == "response" => event,
                NormalizedEvent::Compaction(event) if expected == "compact" => event,
                NormalizedEvent::Notification(event) if expected == "notification" => event,
                NormalizedEvent::HookMark(event) if expected == "hook" => event,
                event => panic!("unexpected event for {event_name}: {event:?}"),
            };
            assert_eq!(session.session_id, "header-session");
            assert_eq!(session.metadata["model"], json!("model-a"));
            assert_eq!(session.metadata["cwd"], json!("/repo"));
            assert_eq!(
                session.metadata["sidecar_config_profile"],
                json!("coverage")
            );
        }
    }

    #[test]
    fn extracts_tool_fields_from_fallback_payload_shapes() {
        let headers = HeaderMap::new();
        let outcome = codex::adapt(
            json!({
                "conversationId": "conversation-1",
                "event": "toolEnded",
                "tool": { "id": "tool-id", "name": "Shell" },
                "arguments": { "cmd": "pwd" },
                "result": { "stdout": "/repo" },
                "permission": "allow"
            }),
            &headers,
        );

        match &outcome.events[0] {
            NormalizedEvent::ToolEnded(event) => {
                assert_eq!(event.session_id, "conversation-1");
                assert_eq!(event.tool_call_id, "tool-id");
                assert_eq!(event.tool_name, "Shell");
                assert_eq!(event.arguments, json!({ "cmd": "pwd" }));
                assert_eq!(event.result, json!({ "stdout": "/repo" }));
                assert_eq!(event.status.as_deref(), Some("allow"));
            }
            event => panic!("unexpected event: {event:?}"),
        }
    }

    #[test]
    fn generated_ids_are_used_when_payload_omits_identifiers() {
        let headers = HeaderMap::new();
        let outcome = claude_code::adapt(
            json!({
                "hook_event_name": "PreToolUse",
                "tool_input": { "name": "Read", "file_path": "Cargo.toml" }
            }),
            &headers,
        );

        match &outcome.events[0] {
            NormalizedEvent::ToolStarted(event) => {
                assert!(event.session_id.starts_with("hook-"));
                assert!(event.tool_call_id.starts_with("tool-"));
                assert_eq!(event.tool_name, "Read");
            }
            event => panic!("unexpected event: {event:?}"),
        }
    }

    #[test]
    fn stop_responses_preserve_vendor_shapes() {
        let headers = HeaderMap::new();
        let claude = claude_code::adapt(
            json!({
                "session_id": "claude-session",
                "hook_event_name": "Stop"
            }),
            &headers,
        );
        assert!(matches!(claude.events[0], NormalizedEvent::AgentEnded(_)));
        assert_eq!(claude.response["stopReason"], Value::Null);

        let cursor = cursor::adapt(
            json!({
                "session_id": "cursor-session",
                "hook_event_name": "stop"
            }),
            &headers,
        );
        assert!(matches!(cursor.events[0], NormalizedEvent::AgentEnded(_)));
        assert_eq!(cursor.response, json!({ "continue": true }));
    }
}
