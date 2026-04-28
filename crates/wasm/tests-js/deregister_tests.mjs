// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { expectAlreadyExists, globalRegistrationCases, unique } from './test_support.mjs';

test('global register and deregister wrappers are callable', () => {
  for (const [prefix, register, deregister, invoke] of globalRegistrationCases()) {
    const name = unique(prefix);
    invoke(name, register);
    assert.equal(deregister(name), true, `${prefix} should deregister`);
    assert.equal(deregister(name), false, `${prefix} should not deregister twice`);
  }
});

test('global registration wrappers reject duplicate names', () => {
  for (const [prefix, register, deregister, invoke] of globalRegistrationCases()) {
    const name = unique(`${prefix}_dup`);
    invoke(name, register);
    expectAlreadyExists(() => invoke(name, register));
    assert.equal(deregister(name), true, `${prefix} duplicate registration should clean up`);
  }
});
