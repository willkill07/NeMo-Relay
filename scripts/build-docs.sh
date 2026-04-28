#!/usr/bin/env bash
# SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export REPO_ROOT

usage() {
    cat <<'EOF'
Usage: ./scripts/build-docs.sh [html|linkcheck|pages]

Compatibility wrapper around the `just` docs recipes.

Modes:
- html       Run `just docs`
- linkcheck  Run `just docs-linkcheck`
- pages      Run `just docs-github-pages`

EOF
    return $?
}

mode="docs"

while [[ $# -gt 0 ]]; do
    case "$1" in
        html)
            mode="docs"
            ;;
        linkcheck)
            mode="docs-linkcheck"
            ;;
        pages)
            mode="docs-github-pages"
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            usage >&2
            exit 1
            ;;
    esac
    shift
done

cd "$REPO_ROOT"
exec just "$mode"
