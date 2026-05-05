// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use reqwest::Client;
use serde_json::Value;
use tokio::net::TcpListener;

use crate::adapters::{claude_code, codex, cursor};
use crate::config::SidecarConfig;
use crate::error::SidecarError;
use crate::gateway;
use crate::session::SessionManager;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: SidecarConfig,
    pub(crate) http: Client,
    pub(crate) sessions: SessionManager,
}

pub(crate) async fn serve(config: SidecarConfig) -> Result<(), SidecarError> {
    let listener = TcpListener::bind(config.bind).await?;
    let app = router(config);
    axum::serve(listener, app).await?;
    Ok(())
}

pub(crate) fn router(config: SidecarConfig) -> Router {
    let sessions = SessionManager::new(config.clone());
    let state = AppState {
        config,
        http: Client::new(),
        sessions,
    };
    Router::new()
        .route("/hooks/codex", post(codex_hook))
        .route("/hooks/claude-code", post(claude_code_hook))
        .route("/hooks/cursor", post(cursor_hook))
        .route("/v1/responses", post(gateway::passthrough))
        .route("/v1/chat/completions", post(gateway::passthrough))
        .route("/v1/messages", post(gateway::passthrough))
        .route("/v1/messages/count_tokens", post(gateway::passthrough))
        .route("/v1/models", get(gateway::models))
        .with_state(state)
}

async fn codex_hook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, SidecarError> {
    let outcome = codex::adapt(payload, &headers);
    state
        .sessions
        .apply_events(&headers, outcome.events)
        .await?;
    Ok(Json(outcome.response))
}

async fn claude_code_hook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, SidecarError> {
    let outcome = claude_code::adapt(payload, &headers);
    state
        .sessions
        .apply_events(&headers, outcome.events)
        .await?;
    Ok(Json(outcome.response))
}

async fn cursor_hook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, SidecarError> {
    let outcome = cursor::adapt(payload, &headers);
    state
        .sessions
        .apply_events(&headers, outcome.events)
        .await?;
    Ok(Json(outcome.response))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::response::IntoResponse;
    use bytes::Bytes;
    use futures_util::stream;
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use super::*;

    fn test_config() -> SidecarConfig {
        SidecarConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            openai_base_url: "http://127.0.0.1".into(),
            anthropic_base_url: "http://127.0.0.1".into(),
            atif_dir: None,
            openinference_endpoint: None,
        }
    }

    #[tokio::test]
    async fn codex_hook_keeps_codex_response_shape() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hooks/codex")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "session_id": "codex-1",
                            "hook_event_name": "sessionStart"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body, json!({}));
    }

    #[tokio::test]
    async fn claude_code_hook_returns_continue_shape() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hooks/claude-code")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "session_id": "claude-1",
                            "hook_event_name": "SessionStart"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["continue"], json!(true));
    }

    #[tokio::test]
    async fn cursor_hook_returns_cursor_permission_fields() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hooks/cursor")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "session_id": "cursor-1",
                            "hook_event_name": "beforeShellExecution",
                            "tool_call_id": "shell-1",
                            "tool_name": "shell",
                            "input": { "command": "pwd" }
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["continue"], json!(true));
        assert_eq!(body["permission"], json!("allow"));
    }

    #[tokio::test]
    async fn gateway_forwards_openai_json_without_rewriting_payload() {
        let upstream = spawn_upstream(false).await;
        let mut config = test_config();
        config.openai_base_url = upstream;
        let app = router(config);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/chat/completions")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer test")
                    .header("connection", "close")
                    .body(Body::from(
                        json!({
                            "model": "gpt-test",
                            "messages": [{ "role": "user", "content": "hello" }]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["model"], json!("gpt-test"));
        assert_eq!(body["authorization"], json!("Bearer test"));
        assert_eq!(body["connection"], Value::Null);
    }

    #[tokio::test]
    async fn gateway_preserves_streaming_body() {
        let upstream = spawn_upstream(true).await;
        let mut config = test_config();
        config.openai_base_url = upstream;
        let app = router(config);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/responses")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "model": "gpt-test",
                            "input": "hello",
                            "stream": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/event-stream"
        );
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(bytes, Bytes::from_static(b"data: one\n\ndata: two\n\n"));
    }

    #[tokio::test]
    async fn gateway_rejects_unsupported_paths() {
        let app = router(test_config());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/unsupported")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn models_route_forwards_get_requests() {
        let upstream = spawn_models_upstream().await;
        let mut config = test_config();
        config.openai_base_url = upstream;
        let app = router(config);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/models?limit=1")
                    .header("authorization", "Bearer test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["path"], json!("/v1/models?limit=1"));
        assert_eq!(body["authorization"], json!("Bearer test"));
    }

    async fn spawn_upstream(streaming: bool) -> String {
        async fn chat(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
            let payload: Value = serde_json::from_slice(&body).unwrap();
            Json(json!({
                "model": payload["model"],
                "authorization": headers
                    .get(header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok()),
                "connection": headers
                    .get(header::CONNECTION)
                    .and_then(|value| value.to_str().ok())
            }))
        }

        async fn stream_response() -> impl IntoResponse {
            let chunks = stream::iter([
                Ok::<_, std::convert::Infallible>(Bytes::from_static(b"data: one\n\n")),
                Ok(Bytes::from_static(b"data: two\n\n")),
            ]);
            (
                [(header::CONTENT_TYPE, "text/event-stream")],
                Body::from_stream(chunks),
            )
        }

        let app = if streaming {
            Router::new().route("/v1/responses", post(stream_response))
        } else {
            Router::new().route("/v1/chat/completions", post(chat))
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{address}")
    }

    async fn spawn_models_upstream() -> String {
        async fn models(headers: HeaderMap, request: Request<Body>) -> impl IntoResponse {
            Json(json!({
                "path": request.uri().path_and_query().map(|value| value.as_str()),
                "authorization": headers
                    .get(header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
            }))
        }

        let app = Router::new().route("/v1/models", get(models));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{address}")
    }
}
