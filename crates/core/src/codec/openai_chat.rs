// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Built-in codec for the OpenAI Chat Completions API.
//!
//! Implements [`LlmCodec`] (request decode/encode) and [`LlmResponseCodec`]
//! (response decode) for the OpenAI Chat Completions format.

use serde::Deserialize;

use crate::api::llm::LlmRequest;
use crate::error::{FlowError, Result};
use crate::json::Json;

use super::request::{AnnotatedLlmRequest, GenerationParams, Message, ToolChoice, ToolDefinition};
use super::response::{
    AnnotatedLlmResponse, ApiSpecificResponse, FinishReason, ResponseToolCall, Usage,
};
use super::traits::{LlmCodec, LlmResponseCodec};

// ---------------------------------------------------------------------------
// Public codec struct
// ---------------------------------------------------------------------------

/// Built-in codec for the OpenAI Chat Completions API.
pub struct OpenAIChatCodec;

// ---------------------------------------------------------------------------
// Private intermediate serde structs for response decode
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawChatCompletion {
    id: Option<String>,
    model: Option<String>,
    choices: Option<Vec<RawChoice>>,
    usage: Option<RawChatUsage>,
    system_fingerprint: Option<String>,
    service_tier: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Json>,
}

#[derive(Deserialize)]
struct RawChoice {
    message: Option<RawMessage>,
    finish_reason: Option<String>,
    logprobs: Option<Json>,
}

#[derive(Deserialize)]
struct RawMessage {
    content: Option<String>,
    tool_calls: Option<Vec<RawToolCall>>,
}

#[derive(Deserialize)]
struct RawToolCall {
    id: Option<String>,
    function: Option<RawFunction>,
}

#[derive(Deserialize)]
struct RawFunction {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Deserialize)]
struct RawChatUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
    prompt_tokens_details: Option<RawPromptTokensDetails>,
}

#[derive(Deserialize)]
struct RawPromptTokensDetails {
    cached_tokens: Option<u64>,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Map OpenAI Chat finish_reason string to normalized [`FinishReason`].
fn map_chat_finish_reason(reason: &str) -> FinishReason {
    match reason {
        "stop" => FinishReason::Complete,
        "length" => FinishReason::Length,
        "tool_calls" | "function_call" => FinishReason::ToolUse,
        "content_filter" => FinishReason::ContentFilter,
        other => FinishReason::Unknown(other.to_string()),
    }
}

/// Parse OpenAI tool call arguments from JSON string to [`Json`] value.
///
/// Falls back to [`Json::String`] if parsing fails (malformed model output).
fn parse_arguments(arguments: &str) -> Json {
    serde_json::from_str(arguments).unwrap_or_else(|_| Json::String(arguments.to_string()))
}

/// Keys that are modeled in [`AnnotatedLlmRequest`] and should NOT go into `extra`.
const MODELED_REQUEST_KEYS: &[&str] = &[
    "messages",
    "model",
    "temperature",
    "max_tokens",
    "max_completion_tokens",
    "top_p",
    "stop",
    "tools",
    "tool_choice",
];

// ---------------------------------------------------------------------------
// LlmResponseCodec implementation
// ---------------------------------------------------------------------------

impl LlmResponseCodec for OpenAIChatCodec {
    fn decode_response(&self, response: &Json) -> Result<AnnotatedLlmResponse> {
        let raw: RawChatCompletion = serde_json::from_value(response.clone())
            .map_err(|e| FlowError::Internal(format!("OpenAI Chat response decode: {e}")))?;

        // Extract first choice (if any).
        let choice = raw.choices.as_ref().and_then(|c| c.first());

        // Map message content.
        let message = choice
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.as_ref())
            .map(|s| super::request::MessageContent::Text(s.clone()));

        // Map tool calls, skipping entries that lack a usable function body.
        // Some providers (proxies, vLLM, NIM) may return partial tool_calls
        // entries where `function` or `function.name` is absent or null.
        let tool_calls = choice
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.tool_calls.as_ref())
            .map(|tcs| {
                tcs.iter()
                    .filter_map(|tc| {
                        let func = tc.function.as_ref()?;
                        let name = func.name.as_ref()?;
                        Some(ResponseToolCall {
                            id: tc.id.clone().unwrap_or_default(),
                            name: name.clone(),
                            arguments: func
                                .arguments
                                .as_deref()
                                .map(parse_arguments)
                                .unwrap_or(Json::Object(Default::default())),
                        })
                    })
                    .collect::<Vec<_>>()
            });

        // Map finish reason.
        let finish_reason = choice
            .and_then(|c| c.finish_reason.as_deref())
            .map(map_chat_finish_reason);

        // Map usage.
        let usage = raw.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            cache_read_tokens: u.prompt_tokens_details.and_then(|d| d.cached_tokens),
            cache_write_tokens: None,
        });

        // Build API-specific fields.
        let logprobs = choice.and_then(|c| c.logprobs.clone());
        let api_specific = Some(ApiSpecificResponse::OpenAIChat {
            logprobs,
            system_fingerprint: raw.system_fingerprint,
            service_tier: raw.service_tier,
        });

        Ok(AnnotatedLlmResponse {
            id: raw.id,
            model: raw.model,
            message,
            tool_calls,
            finish_reason,
            usage,
            api_specific,
            extra: raw.extra,
        })
    }
}

// ---------------------------------------------------------------------------
// LlmCodec implementation
// ---------------------------------------------------------------------------

impl LlmCodec for OpenAIChatCodec {
    fn decode(&self, request: &LlmRequest) -> Result<AnnotatedLlmRequest> {
        let obj = request
            .content
            .as_object()
            .ok_or_else(|| FlowError::Internal("request content is not an object".into()))?;

        // Extract messages (default to empty vec if absent).
        let messages: Vec<Message> = obj
            .get("messages")
            .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
            .unwrap_or_default();

        // Extract model.
        let model = obj.get("model").and_then(|v| v.as_str()).map(String::from);

        // Extract generation params.
        let temperature = obj.get("temperature").and_then(|v| v.as_f64());
        let top_p = obj.get("top_p").and_then(|v| v.as_f64());
        let stop = obj
            .get("stop")
            .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok());

        // max_completion_tokens takes priority over max_tokens (newer API key).
        let max_tokens = obj
            .get("max_completion_tokens")
            .and_then(|v| v.as_u64())
            .or_else(|| obj.get("max_tokens").and_then(|v| v.as_u64()));

        let params =
            if temperature.is_some() || max_tokens.is_some() || top_p.is_some() || stop.is_some() {
                Some(GenerationParams {
                    temperature,
                    max_tokens,
                    top_p,
                    stop,
                })
            } else {
                None
            };

        // Extract tools.
        let tools: Option<Vec<ToolDefinition>> = obj
            .get("tools")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
            .map_err(|e| FlowError::Internal(format!("OpenAI Chat tools decode: {e}")))?;

        // Extract tool_choice.
        let tool_choice: Option<ToolChoice> = obj
            .get("tool_choice")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
            .map_err(|e| FlowError::Internal(format!("OpenAI Chat tool_choice decode: {e}")))?;

        // Collect extra fields (keys not in MODELED_REQUEST_KEYS).
        let extra: serde_json::Map<String, Json> = obj
            .iter()
            .filter(|(k, _)| !MODELED_REQUEST_KEYS.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(AnnotatedLlmRequest {
            messages,
            model,
            params,
            tools,
            tool_choice,
            extra,
        })
    }

    fn encode(&self, annotated: &AnnotatedLlmRequest, original: &LlmRequest) -> Result<LlmRequest> {
        let mut content = original.content.clone();
        let obj = content
            .as_object_mut()
            .ok_or_else(|| FlowError::Internal("original content is not an object".into()))?;

        insert_serialized(obj, "messages", &annotated.messages, "messages")?;

        if let Some(ref model) = annotated.model {
            obj.insert("model".into(), Json::String(model.clone()));
        }

        if let Some(ref params) = annotated.params {
            overlay_generation_params(obj, params)?;
        }

        if let Some(ref tools) = annotated.tools {
            insert_serialized(obj, "tools", tools, "tools")?;
        }

        if let Some(ref tool_choice) = annotated.tool_choice {
            insert_serialized(obj, "tool_choice", tool_choice, "tool_choice")?;
        }

        for (k, v) in &annotated.extra {
            obj.insert(k.clone(), v.clone());
        }

        // Force `stream_options.include_usage` when the caller did not set it.
        //
        // Rationale: OpenAI-compatible backends only emit the terminal chunk
        // containing `usage` (prompt/completion/total tokens) when this flag
        // is true. Without it, Phoenix spans show `token_count=0` for every
        // LLM call even though the provider knows the real counts. The
        // observability exporter (OpenInference) reads usage off the
        // annotated response, so the flag has to be set at the request level
        // before bytes go on the wire.
        //
        // Guarded on `stream == true` per the OpenAI Chat Completions spec,
        // which restricts `stream_options` to streaming requests. Caller-
        // provided `stream_options` are preserved verbatim (including
        // explicit opt-outs such as `include_usage: false`).
        let is_streaming = obj.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_streaming && !obj.contains_key("stream_options") {
            obj.insert(
                "stream_options".into(),
                serde_json::json!({"include_usage": true}),
            );
        }

        Ok(LlmRequest {
            headers: original.headers.clone(),
            content,
        })
    }
}

/// Helper to construct a [`Json`] number from an `f64`.
fn json_f64(v: f64) -> Json {
    serde_json::Number::from_f64(v)
        .map(Json::Number)
        .unwrap_or(Json::Null)
}

fn insert_serialized<T: serde::Serialize>(
    obj: &mut serde_json::Map<String, Json>,
    key: &str,
    value: &T,
    context: &str,
) -> Result<()> {
    let json = serde_json::to_value(value)
        .map_err(|e| FlowError::Internal(format!("OpenAI Chat {context} encode: {e}")))?;
    obj.insert(key.into(), json);
    Ok(())
}

fn overlay_generation_params(
    obj: &mut serde_json::Map<String, Json>,
    params: &GenerationParams,
) -> Result<()> {
    if let Some(temp) = params.temperature {
        obj.insert("temperature".into(), json_f64(temp));
    }
    if let Some(top_p) = params.top_p {
        obj.insert("top_p".into(), json_f64(top_p));
    }
    if let Some(ref stop) = params.stop {
        insert_serialized(obj, "stop", stop, "stop")?;
    }
    if let Some(max_tokens) = params.max_tokens {
        let key = if obj.contains_key("max_completion_tokens") {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        obj.insert(key.into(), Json::from(max_tokens));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "../../tests/unit/codec/openai_chat_tests.rs"]
mod tests;
