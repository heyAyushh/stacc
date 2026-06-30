---
title: Architecture
eyebrow: Core System
description: STACC organizes agent behavior, rules, hooks, and install targets into a predictable configuration pipeline for local and global coding environments.
order: 1
sections:
  - id: config-surface
    label: Config Surface
  - id: install-planner
    label: Install Planner
  - id: merge-boundaries
    label: Merge Boundaries
---

STACC is split between the Rust control plane and the checked-in configuration payload.
