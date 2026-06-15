// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package pricing

import "testing"

func TestPricingPackageHelpers(t *testing.T) {
	entry := NewModelPricing("test", "priced-model")
	entry.PricingAsOf = "2026-06-15"
	entry.PricingSource = "https://example.com/pricing"
	rates := NewTokenRates(1, 2)
	entry.Rates = &rates

	config := NewConfig()
	config.Sources = []SourceConfig{
		NewInlineSource(NewCatalog(entry)),
	}
	component := NewComponentSpec(config).PluginComponent()
	if component.Kind != PluginKind {
		t.Fatalf("unexpected pricing component kind: %#v", component)
	}

	report, err := ValidateConfig(config)
	if err != nil {
		t.Fatalf("ValidateConfig failed: %v", err)
	}
	if len(report.Diagnostics) != 0 {
		t.Fatalf("expected clean report, got %#v", report.Diagnostics)
	}
}
