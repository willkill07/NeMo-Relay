// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::http::HeaderMap;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::Value;

#[derive(Debug, Clone, Parser)]
#[command(name = "nemo-flow-sidecar")]
#[command(about = "Gateway sidecar for coding-agent NeMo Flow observability")]
pub(crate) struct Cli {
    #[command(flatten)]
    pub(crate) server: SidecarConfig,
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum Command {
    Install(InstallCommand),
    HookForward(HookForwardCommand),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct SidecarConfig {
    #[arg(long, env = "NEMO_FLOW_SIDECAR_BIND", default_value = "127.0.0.1:4040")]
    pub(crate) bind: SocketAddr,
    #[arg(
        long,
        env = "NEMO_FLOW_OPENAI_BASE_URL",
        default_value = "https://api.openai.com"
    )]
    pub(crate) openai_base_url: String,
    #[arg(
        long,
        env = "NEMO_FLOW_ANTHROPIC_BASE_URL",
        default_value = "https://api.anthropic.com"
    )]
    pub(crate) anthropic_base_url: String,
    #[arg(long, env = "NEMO_FLOW_ATIF_DIR")]
    pub(crate) atif_dir: Option<PathBuf>,
    #[arg(long, env = "NEMO_FLOW_OPENINFERENCE_ENDPOINT")]
    pub(crate) openinference_endpoint: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct InstallCommand {
    #[arg(value_enum)]
    pub(crate) agent: CodingAgent,
    #[arg(long, value_enum, default_value = "user")]
    pub(crate) scope: InstallScope,
    #[arg(long, value_enum, default_value = "both")]
    pub(crate) target: InstallTarget,
    #[arg(long, default_value = "http://127.0.0.1:4040")]
    pub(crate) sidecar_url: String,
    #[arg(long)]
    pub(crate) atif_dir: Option<PathBuf>,
    #[arg(long)]
    pub(crate) openinference_endpoint: Option<String>,
    #[arg(long)]
    pub(crate) profile: Option<String>,
    #[arg(long)]
    pub(crate) session_metadata: Option<String>,
    #[arg(long)]
    pub(crate) plugin_config: Option<String>,
    #[arg(long, value_enum)]
    pub(crate) gateway_mode: Option<GatewayMode>,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) print: bool,
    #[arg(long, hide = true)]
    pub(crate) home_dir: Option<PathBuf>,
    #[arg(long, hide = true)]
    pub(crate) project_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HookForwardCommand {
    #[arg(value_enum)]
    pub(crate) agent: CodingAgent,
    #[arg(long, default_value = "http://127.0.0.1:4040")]
    pub(crate) sidecar_url: String,
    #[arg(long)]
    pub(crate) atif_dir: Option<PathBuf>,
    #[arg(long)]
    pub(crate) openinference_endpoint: Option<String>,
    #[arg(long)]
    pub(crate) profile: Option<String>,
    #[arg(long)]
    pub(crate) session_metadata: Option<String>,
    #[arg(long)]
    pub(crate) plugin_config: Option<String>,
    #[arg(long, value_enum)]
    pub(crate) gateway_mode: Option<GatewayMode>,
    #[arg(long)]
    pub(crate) fail_closed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum CodingAgent {
    ClaudeCode,
    Codex,
    Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum InstallScope {
    User,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum InstallTarget {
    Cli,
    Gui,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum GatewayMode {
    HookOnly,
    Passthrough,
    Required,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SessionConfig {
    pub(crate) atif_dir: Option<PathBuf>,
    pub(crate) openinference_endpoint: Option<String>,
    pub(crate) metadata: Option<Value>,
    pub(crate) plugin_config: Option<Value>,
    pub(crate) profile: Option<String>,
    pub(crate) gateway_mode: Option<String>,
}

impl SidecarConfig {
    pub(crate) fn session_config_from_headers(&self, headers: &HeaderMap) -> SessionConfig {
        let atif_dir = header_string(headers, "x-nemo-flow-atif-dir")
            .map(PathBuf::from)
            .or_else(|| self.atif_dir.clone());
        let openinference_endpoint = header_string(headers, "x-nemo-flow-openinference-endpoint")
            .or_else(|| self.openinference_endpoint.clone());
        let metadata = header_json(headers, "x-nemo-flow-session-metadata");
        let plugin_config = header_json(headers, "x-nemo-flow-plugin-config");
        let profile = header_string(headers, "x-nemo-flow-config-profile");
        let gateway_mode = header_string(headers, "x-nemo-flow-gateway-mode");
        SessionConfig {
            atif_dir,
            openinference_endpoint,
            metadata,
            plugin_config,
            profile,
            gateway_mode,
        }
    }
}

pub(crate) fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn header_json(headers: &HeaderMap, name: &str) -> Option<Value> {
    header_string(headers, name).and_then(|raw| serde_json::from_str(&raw).ok())
}

impl CodingAgent {
    pub(crate) const fn hook_path(self) -> &'static str {
        match self {
            Self::ClaudeCode => "/hooks/claude-code",
            Self::Codex => "/hooks/codex",
            Self::Cursor => "/hooks/cursor",
        }
    }

    pub(crate) const fn as_arg(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
        }
    }
}

impl GatewayMode {
    pub(crate) const fn as_arg(self) -> &'static str {
        match self {
            Self::HookOnly => "hook-only",
            Self::Passthrough => "passthrough",
            Self::Required => "required",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use serde_json::json;

    fn config() -> SidecarConfig {
        SidecarConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            openai_base_url: "http://openai".into(),
            anthropic_base_url: "http://anthropic".into(),
            atif_dir: Some(PathBuf::from("default-atif")),
            openinference_endpoint: Some("http://default-otel".into()),
        }
    }

    #[test]
    fn session_config_prefers_headers_and_parses_json() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-nemo-flow-atif-dir",
            HeaderValue::from_static("header-atif"),
        );
        headers.insert(
            "x-nemo-flow-openinference-endpoint",
            HeaderValue::from_static("http://header-otel"),
        );
        headers.insert(
            "x-nemo-flow-config-profile",
            HeaderValue::from_static("profile-a"),
        );
        headers.insert(
            "x-nemo-flow-session-metadata",
            HeaderValue::from_static(r#"{"team":"obs"}"#),
        );
        headers.insert(
            "x-nemo-flow-plugin-config",
            HeaderValue::from_static(r#"{"components":[]}"#),
        );
        headers.insert(
            "x-nemo-flow-gateway-mode",
            HeaderValue::from_static("required"),
        );

        let session = config().session_config_from_headers(&headers);

        assert_eq!(session.atif_dir, Some(PathBuf::from("header-atif")));
        assert_eq!(
            session.openinference_endpoint.as_deref(),
            Some("http://header-otel")
        );
        assert_eq!(session.profile.as_deref(), Some("profile-a"));
        assert_eq!(session.metadata, Some(json!({ "team": "obs" })));
        assert_eq!(session.plugin_config, Some(json!({ "components": [] })));
        assert_eq!(session.gateway_mode.as_deref(), Some("required"));
    }

    #[test]
    fn session_config_uses_defaults_and_ignores_bad_json() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-nemo-flow-session-metadata",
            HeaderValue::from_static("not-json"),
        );
        headers.insert("x-empty", HeaderValue::from_static(""));

        let session = config().session_config_from_headers(&headers);

        assert_eq!(session.atif_dir, Some(PathBuf::from("default-atif")));
        assert_eq!(
            session.openinference_endpoint.as_deref(),
            Some("http://default-otel")
        );
        assert_eq!(session.metadata, None);
        assert_eq!(header_string(&headers, "x-empty"), None);
    }

    #[test]
    fn agent_and_gateway_mode_arguments_are_stable() {
        assert_eq!(CodingAgent::ClaudeCode.hook_path(), "/hooks/claude-code");
        assert_eq!(CodingAgent::Codex.hook_path(), "/hooks/codex");
        assert_eq!(CodingAgent::Cursor.hook_path(), "/hooks/cursor");
        assert_eq!(GatewayMode::HookOnly.as_arg(), "hook-only");
        assert_eq!(GatewayMode::Passthrough.as_arg(), "passthrough");
        assert_eq!(GatewayMode::Required.as_arg(), "required");
    }
}
