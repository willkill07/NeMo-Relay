// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use axum::http::HeaderMap;
use serde_json::{Value, json};

use crate::adapters::{AdapterOutcome, ClassificationRules, classify, event_name, normalize_name};
use crate::model::{AgentKind, NormalizedEvent};

pub(crate) fn adapt(payload: Value, headers: &HeaderMap) -> AdapterOutcome {
    let event = classify(
        &payload,
        headers,
        &ClassificationRules {
            kind: AgentKind::ClaudeCode,
            agent_start: &["SessionStart", "sessionStart", "session_start"],
            agent_end: &["SessionEnd", "sessionEnd", "session_end"],
            subagent_start: &["SubagentStart", "subagentStart"],
            subagent_end: &["SubagentStop", "subagentStop", "SubagentEnd"],
            tool_start: &["PreToolUse", "preToolUse"],
            tool_end: &[
                "PostToolUse",
                "postToolUse",
                "PostToolUseFailure",
                "postToolUseFailure",
                "ToolUseFailed",
                "toolUseFailed",
                "PermissionDenied",
                "permissionDenied",
            ],
        },
    );
    let normalized_event = normalize_name(&event_name(&payload));
    let response = match &event {
        NormalizedEvent::ToolStarted(_) => json!({
            "continue": true,
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow"
            }
        }),
        NormalizedEvent::AgentEnded(_) | NormalizedEvent::HookMark(_)
            if normalized_event == "stop" =>
        {
            json!({ "continue": true, "stopReason": null })
        }
        NormalizedEvent::AgentEnded(_) => json!({ "continue": true }),
        _ => json!({ "continue": true }),
    };
    AdapterOutcome {
        events: vec![event],
        response,
    }
}
