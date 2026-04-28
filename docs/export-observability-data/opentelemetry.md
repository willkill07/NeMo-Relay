<!--
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
-->

# Advanced Guide: Export OpenTelemetry Data

Use this guide when you want NeMo Flow lifecycle events exported as generic OpenTelemetry Protocol (OTLP) trace spans.

## What You Build

You will configure the OpenTelemetry subscriber, register it before instrumented work starts, run scoped work, flush spans, and shut down the exporter.

Use OpenTelemetry export when your tracing backend already expects OTLP spans and you want NeMo Flow scopes, tool calls, LLM calls, and marks to appear in the same tracing pipeline as the rest of the application.

## Before You Start

Complete these steps:

1. Instrument at least one scope, tool call, or LLM call.
2. Start or identify an OTLP trace collector endpoint.
3. Decide the `service_name`, namespace, and deployment attributes for this process.
4. Add sanitize guardrails for sensitive event payloads before production export.

For OpenInference-specific span semantics, use [Advanced Guide: Export OpenInference Data](advanced-guide.md).

## Configure the Exporter

Set these fields first:

| Field | Purpose |
|---|---|
| `transport` | OTLP transport. Start with `http_binary`. |
| `endpoint` | OTLP trace endpoint, such as `http://localhost:4318/v1/traces`. |
| `service_name` | Logical service name shown in traces. |
| `service_namespace` | Optional namespace for grouping related services. |
| `service_version` | Optional application or package version. |
| `instrumentation_scope` | Scope name for NeMo Flow spans. |
| `headers` | Optional auth or routing headers. |
| `resource_attributes` | Deployment metadata such as environment or region. |
| `timeout_millis` | Export timeout in milliseconds. |

`OTEL_*` variables may be used by the underlying OpenTelemetry exporter when values are not set directly in config. Prefer explicit config objects in application code so the docs, tests, and deployment manifests show the active export settings.

## Register and Export

The examples below show how to register the exporter and emit data from instrumented
work.

::::{tab-set}
:sync-group: language

:::{tab-item} Python
:sync: python

```python
from nemo_flow import OpenTelemetryConfig, OpenTelemetrySubscriber

config = OpenTelemetryConfig()
config.transport = "http_binary"
config.endpoint = "http://localhost:4318/v1/traces"
config.service_name = "agent"
config.service_namespace = "nemo"
config.service_version = "1.0.0"
config.instrumentation_scope = "nemo-flow-otel"
config.timeout_millis = 3000
config.headers = {"authorization": "Bearer token"}
config.resource_attributes = {"deployment.environment": "dev"}

subscriber = OpenTelemetrySubscriber(config)
subscriber.register("otel-exporter")

# Run instrumented application work here.

subscriber.force_flush()
subscriber.deregister("otel-exporter")
subscriber.shutdown()
```
:::

:::{tab-item} Node.js
:sync: node

```js
const { OpenTelemetrySubscriber } = require("nemo-flow-node");

const subscriber = new OpenTelemetrySubscriber({
  transport: "http_binary",
  endpoint: "http://localhost:4318/v1/traces",
  serviceName: "agent",
  serviceNamespace: "nemo",
  serviceVersion: "1.0.0",
  instrumentationScope: "nemo-flow-otel",
  timeoutMillis: 3000,
  headers: { authorization: "Bearer token" },
  resourceAttributes: { "deployment.environment": "dev" },
});

subscriber.register("otel-exporter");

// Run instrumented application work here.

subscriber.forceFlush();
subscriber.deregister("otel-exporter");
subscriber.shutdown();
```
:::

:::{tab-item} Rust
:sync: rust

```rust
use nemo_flow::observability::otel::{OpenTelemetryConfig, OpenTelemetrySubscriber};

let subscriber = OpenTelemetrySubscriber::new(
    OpenTelemetryConfig::http_binary("agent")
        .with_endpoint("http://localhost:4318/v1/traces")
        .with_header("authorization", "Bearer token")
        .with_resource_attribute("deployment.environment", "dev")
        .with_service_namespace("nemo")
        .with_service_version("1.0.0")
        .with_instrumentation_scope("nemo-flow-otel"),
)?;

subscriber.register("otel-exporter")?;

// Run instrumented application work here.

subscriber.force_flush()?;
subscriber.deregister("otel-exporter")?;
subscriber.shutdown()?;
```
:::

::::

## Validate the Export

Check the export in this order:

1. Application startup should not report exporter construction errors.
2. The collector should receive OTLP trace export requests.
3. The tracing backend should show spans for NeMo Flow scopes, tools, and LLM calls.
4. Span grouping should match the same root scope for one agent run.

If spans arrive without useful payloads, verify that the application uses managed execution helpers or explicit lifecycle APIs. If payloads contain sensitive fields, add sanitize guardrails before registering the exporter in production.

## Production Checklist

Use this checklist before running the pattern in production traffic.

- Register the subscriber before the first instrumented request.
- Use stable service identity across deployments.
- Keep auth headers and endpoints outside source code.
- Flush during graceful shutdown.
- Redact sensitive payloads before export.
- Filter by `root_uuid` when analyzing concurrent agent runs.

## Common Issues

Check these symptoms first when the workflow does not behave as expected.

- **No spans appear**: Confirm the OTLP endpoint is reachable and the application emits events.
- **Only scope spans appear**: Route tools and LLM calls through managed execute helpers or explicit lifecycle APIs.
- **Exporter hangs during shutdown**: Lower `timeout_millis` and make shutdown flush bounded.
- **Sensitive data appears in traces**: Add sanitize-request and sanitize-response guardrails before registering production exporters.

## Next Steps

Use these links to continue from this workflow into the next related task.

- Add custom redaction with [Advanced Guide: Add Middleware](../instrument-applications/advanced-guide.md).
- Compare exporter semantics with [Advanced Guide: Export OpenInference Data](advanced-guide.md).
- Review concrete snippets in [Code Examples](code-examples.md#opentelemetry-export).
