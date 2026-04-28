// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package scope_test

import (
	"encoding/json"
	"testing"

	"github.com/NVIDIA/NeMo-Flow/go/nemo_flow"
	"github.com/NVIDIA/NeMo-Flow/go/nemo_flow/scope"
)

func TestWithScopeCleanupNoopsWhenPushFails(t *testing.T) {
	for _, tc := range []struct {
		name string
		opt  nemo_flow.ScopeOption
	}{
		{name: "data", opt: nemo_flow.WithData(json.RawMessage("{"))},
		{name: "metadata", opt: nemo_flow.WithMetadata(json.RawMessage("{"))},
		{name: "input", opt: nemo_flow.WithInput(json.RawMessage("{"))},
	} {
		before, err := nemo_flow.GetHandle()
		if err != nil {
			t.Fatalf("GetHandle before failed: %v", err)
		}

		cleanup := scope.WithScope("invalid_scope_"+tc.name, nemo_flow.ScopeTypeAgent, tc.opt)
		cleanup()

		after, err := nemo_flow.GetHandle()
		if err != nil {
			t.Fatalf("GetHandle after WithScope failure failed: %v", err)
		}
		if after.UUID() != before.UUID() {
			t.Fatalf("expected top of stack to remain %q after invalid %s, got %q", before.UUID(), tc.name, after.UUID())
		}

		handle, cleanupHandle := scope.WithScopeHandle("invalid_scope_"+tc.name, nemo_flow.ScopeTypeAgent, tc.opt)
		if handle != nil {
			t.Fatalf("expected nil handle on failed push for invalid %s, got %#v", tc.name, handle)
		}
		cleanupHandle()

		afterHandle, err := nemo_flow.GetHandle()
		if err != nil {
			t.Fatalf("GetHandle after WithScopeHandle failure failed: %v", err)
		}
		if afterHandle.UUID() != before.UUID() {
			t.Fatalf("expected top of stack to remain %q after invalid %s, got %q", before.UUID(), tc.name, afterHandle.UUID())
		}
	}
}
