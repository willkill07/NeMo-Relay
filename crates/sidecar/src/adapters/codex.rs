// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use axum::http::HeaderMap;
use serde_json::{Value, json};

use crate::adapters::{AdapterOutcome, ClassificationRules, classify};
use crate::model::AgentKind;

pub(crate) fn adapt(payload: Value, headers: &HeaderMap) -> AdapterOutcome {
    let event = classify(
        &payload,
        headers,
        &ClassificationRules {
            kind: AgentKind::Codex,
            agent_start: &["sessionStart", "session_start", "agentStarted"],
            agent_end: &["sessionEnd", "session_end", "agentEnded"],
            subagent_start: &["subagentStart", "subagent_start"],
            subagent_end: &["subagentStop", "subagentEnd", "subagent_stop"],
            tool_start: &["preToolUse", "toolStarted", "tool_start"],
            tool_end: &["postToolUse", "toolEnded", "tool_end", "toolFailed"],
        },
    );
    AdapterOutcome {
        events: vec![event],
        response: json!({}),
    }
}
