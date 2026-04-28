// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);

// These tests intentionally exercise only the public generated package API.
// Avoid asserting against wasm-bindgen implementation details or private helpers.
export const wasm = require('../pkg');
export const pkgDir = fileURLToPath(new URL('../pkg/', import.meta.url));
export const testsJsDir = fileURLToPath(new URL('.', import.meta.url));
export const SCOPE_ATTR_PARALLEL = 0b01;
export const SCOPE_ATTR_RELOCATABLE = 0b10;

export function unique(prefix) {
  return `${prefix}_${Date.now()}_${Math.random().toString(16).slice(2)}`;
}

export function assertBodyContains(body, text) {
  assert.equal(body.includes(Buffer.from(text, 'utf8')), true, `expected OTLP payload to contain ${text}`);
}

export function resetScopeStack() {
  const stack = wasm.createScopeStack();
  wasm.setThreadScopeStack(stack);
  return stack;
}

export function currentScope() {
  return wasm.getHandle();
}

export function makeLlmRequest(model = 'demo-model') {
  return {
    headers: {},
    content: {
      model,
      messages: [],
    },
  };
}

export async function drainStream(stream) {
  const chunks = [];
  for (;;) {
    const next = await stream.next();
    if (next === null) {
      return chunks;
    }
    chunks.push(next);
  }
}

export async function waitFor(predicate, timeoutMs = 2000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const result = predicate();
    if (result) {
      return result;
    }
    if (Date.now() >= deadline) {
      assert.fail('timed out waiting for condition');
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

export function expectInvalidUuid(fn) {
  assert.throws(fn, /invalid UUID/i);
}

export function expectInvalidLlmRequest(fn) {
  assert.throws(fn, /invalid type|LlmRequest/i);
}

export async function rejectInvalidLlmRequest(promise) {
  await assert.rejects(promise, /invalid type|LlmRequest/i);
}

export function expectClassError(fn) {
  assert.throws(fn, /expected instance of/i);
}

export function expectAlreadyExists(fn) {
  assert.throws(fn, /already exists/i);
}

export function globalRegistrationCases() {
  return [
    [
      'toolSanReq',
      wasm.registerToolSanitizeRequestGuardrail,
      wasm.deregisterToolSanitizeRequestGuardrail,
      (name, register) => register(name, 1, (_toolName, args) => args),
    ],
    [
      'toolSanResp',
      wasm.registerToolSanitizeResponseGuardrail,
      wasm.deregisterToolSanitizeResponseGuardrail,
      (name, register) => register(name, 1, (result) => result),
    ],
    [
      'toolCond',
      wasm.registerToolConditionalExecutionGuardrail,
      wasm.deregisterToolConditionalExecutionGuardrail,
      (name, register) => register(name, 1, () => undefined),
    ],
    [
      'toolReq',
      wasm.registerToolRequestIntercept,
      wasm.deregisterToolRequestIntercept,
      (name, register) => register(name, 1, false, (_toolName, args) => args),
    ],
    [
      'toolExec',
      wasm.registerToolExecutionIntercept,
      wasm.deregisterToolExecutionIntercept,
      (name, register) => register(name, 1, async (args, next) => next(args)),
    ],
    [
      'llmSanReq',
      wasm.registerLlmSanitizeRequestGuardrail,
      wasm.deregisterLlmSanitizeRequestGuardrail,
      (name, register) => register(name, 1, (request) => request),
    ],
    [
      'llmSanResp',
      wasm.registerLlmSanitizeResponseGuardrail,
      wasm.deregisterLlmSanitizeResponseGuardrail,
      (name, register) => register(name, 1, (response) => response),
    ],
    [
      'llmCond',
      wasm.registerLlmConditionalExecutionGuardrail,
      wasm.deregisterLlmConditionalExecutionGuardrail,
      (name, register) => register(name, 1, () => undefined),
    ],
    [
      'llmReq',
      wasm.registerLlmRequestIntercept,
      wasm.deregisterLlmRequestIntercept,
      (name, register) => register(name, 1, false, (request) => request),
    ],
    [
      'llmExec',
      wasm.registerLlmExecutionIntercept,
      wasm.deregisterLlmExecutionIntercept,
      (name, register) => register(name, 1, async (request, next) => next(request)),
    ],
    [
      'llmStreamExec',
      wasm.registerLlmStreamExecutionIntercept,
      wasm.deregisterLlmStreamExecutionIntercept,
      (name, register) => register(name, 1, async (request, next) => next(request)),
    ],
    ['subscriber', wasm.registerSubscriber, wasm.deregisterSubscriber, (name, register) => register(name, () => {})],
  ];
}

export function scopeRegistrationCases(scopeUuid) {
  return [
    [
      'scopeToolSanReq',
      wasm.scopeRegisterToolSanitizeRequestGuardrail,
      wasm.scopeDeregisterToolSanitizeRequestGuardrail,
      (uuid, name, register) => register(uuid, name, 1, (_toolName, args) => args),
    ],
    [
      'scopeToolSanResp',
      wasm.scopeRegisterToolSanitizeResponseGuardrail,
      wasm.scopeDeregisterToolSanitizeResponseGuardrail,
      (uuid, name, register) => register(uuid, name, 1, (result) => result),
    ],
    [
      'scopeToolCond',
      wasm.scopeRegisterToolConditionalExecutionGuardrail,
      wasm.scopeDeregisterToolConditionalExecutionGuardrail,
      (uuid, name, register) => register(uuid, name, 1, () => undefined),
    ],
    [
      'scopeToolReq',
      wasm.scopeRegisterToolRequestIntercept,
      wasm.scopeDeregisterToolRequestIntercept,
      (uuid, name, register) => register(uuid, name, 1, false, (_toolName, args) => args),
    ],
    [
      'scopeToolExec',
      wasm.scopeRegisterToolExecutionIntercept,
      wasm.scopeDeregisterToolExecutionIntercept,
      (uuid, name, register) => register(uuid, name, 1, async (args, next) => next(args)),
    ],
    [
      'scopeLlmSanReq',
      wasm.scopeRegisterLlmSanitizeRequestGuardrail,
      wasm.scopeDeregisterLlmSanitizeRequestGuardrail,
      (uuid, name, register) => register(uuid, name, 1, (request) => request),
    ],
    [
      'scopeLlmSanResp',
      wasm.scopeRegisterLlmSanitizeResponseGuardrail,
      wasm.scopeDeregisterLlmSanitizeResponseGuardrail,
      (uuid, name, register) => register(uuid, name, 1, (response) => response),
    ],
    [
      'scopeLlmCond',
      wasm.scopeRegisterLlmConditionalExecutionGuardrail,
      wasm.scopeDeregisterLlmConditionalExecutionGuardrail,
      (uuid, name, register) => register(uuid, name, 1, () => undefined),
    ],
    [
      'scopeLlmReq',
      wasm.scopeRegisterLlmRequestIntercept,
      wasm.scopeDeregisterLlmRequestIntercept,
      (uuid, name, register) => register(uuid, name, 1, false, (request) => request),
    ],
    [
      'scopeLlmExec',
      wasm.scopeRegisterLlmExecutionIntercept,
      wasm.scopeDeregisterLlmExecutionIntercept,
      (uuid, name, register) => register(uuid, name, 1, async (request, next) => next(request)),
    ],
    [
      'scopeLlmStreamExec',
      wasm.scopeRegisterLlmStreamExecutionIntercept,
      wasm.scopeDeregisterLlmStreamExecutionIntercept,
      (uuid, name, register) => register(uuid, name, 1, async (request, next) => next(request)),
    ],
    [
      'scopeSubscriber',
      wasm.scopeRegisterSubscriber,
      wasm.scopeDeregisterSubscriber,
      (uuid, name, register) => register(uuid, name, () => {}),
    ],
  ];
}
