/*
SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
SPDX-License-Identifier: Apache-2.0
*/

(() => {
  const selector = ".docs-version-switcher";

  const closeOthers = (active) => {
    document.querySelectorAll(`${selector}[open]`).forEach((node) => {
      if (node !== active) {
        node.removeAttribute("open");
      }
    });
  };

  document.addEventListener("toggle", (event) => {
    const node = event.target;
    if (!(node instanceof HTMLElement) || !node.matches(selector) || !node.open) {
      return;
    }
    closeOthers(node);
  });

  document.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof Node)) {
      return;
    }

    document.querySelectorAll(`${selector}[open]`).forEach((node) => {
      if (!node.contains(target)) {
        node.removeAttribute("open");
      }
    });
  });

  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") {
      return;
    }

    const openSwitcher = document.querySelector(`${selector}[open]`);
    if (!(openSwitcher instanceof HTMLDetailsElement)) {
      return;
    }

    openSwitcher.removeAttribute("open");
    openSwitcher.querySelector("summary")?.focus();
  });
})();
