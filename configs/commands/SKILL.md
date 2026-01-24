---
name: commands
description: Overview of Cursor slash commands available in this repo and how to find their detailed workflows.
---

# Commands Skill

## Overview

This skill provides access to a collection of slash commands for Cursor IDE. These commands are markdown files that define workflows, prompts, and instructions for common development tasks. Each command can be invoked directly in Cursor's chat interface.

## Quick Reference

Commands are organized by category below. Each command file contains detailed instructions. Read the specific command file when you need to use it.

## Code Review & Analysis

### `/review-pr`
Comprehensive code review for pull requests with multi-agent validation, CLAUDE.md compliance checking, and inline comment posting. See [review-pr.md](review-pr.md) for the complete workflow.

### `/review`
Security-focused code review analyzing changes for exploitable vulnerabilities. See [review.md](review.md) for security categories and analysis methodology.

## Git Operations

### `/commit`
Simple commit template for current branch changes. See [commit.md](commit.md) for commit message format guidelines.

### `/commit-push`
Commit and push changes to the current branch. See [commit-push.md](commit-push.md) for workflow steps.

### `/commit-push-pr`
One-shot workflow: commit, push, and create a pull request. See [commit-push-pr.md](commit-push-pr.md) for the complete automation flow.

### `/rebase`
Rebase current branch. See [rebase.md](rebase.md) for rebase instructions.

### `/clean-gone`
Clean up branches that no longer exist on remote. See [clean-gone.md](clean-gone.md) for cleanup workflow.

## Code Quality & Refactoring

### `/refactor`
Refactor code to improve quality while maintaining functionality. See [refactor.md](refactor.md) for refactoring checklist and principles.

### `/simplify`
Simplify code structure and logic. See [simplify.md](simplify.md) for simplification guidelines.

### `/deslop`
Remove AI-generated code slop and improve code quality. See [deslop.md](deslop.md) for cleanup patterns.

## Codebase Exploration

### `/explore`
Read-only file search and codebase exploration specialist. See [explore.md](explore.md) for search patterns and guidelines.

### `/visualize`
Generate Mermaid diagrams from codebase structure. See [visualize.md](visualize.md) for diagram generation.

### `/ultrathink`
Deep reasoning mode for complex problems. See [ultrathink.md](ultrathink.md) for deep analysis workflow.

## Development Workflows

### `/council`
Multi-agent exploration and task delegation for complex areas of interest. See [council.md](council.md) for the council workflow.

### `/iterate-browser`
Iterate on tasks using debug instrumentation and browser tools. See [iterate-browser.md](iterate-browser.md) for browser-based debugging workflow.

### `/init`
Initialize AGENTS.md documentation for a codebase. See [init.md](init.md) for documentation generation guidelines.

### `/onboard-new-developer`
Onboard new developers to the project. See [onboard-new-developer.md](onboard-new-developer.md) for onboarding workflow.

## Command Management

### `/create-command`
Create a new command file following this repo's style and structure. See [create-command.md](create-command.md) for command creation template and guidelines.

## Usage Notes

- Commands are markdown files that can be invoked directly in Cursor's chat
- Each command file contains complete instructions and workflows
- Commands may specify allowed tools, descriptions, and step-by-step processes
- Some commands use multi-agent workflows for parallel processing
- Read the specific command file when you need detailed implementation guidance

## Progressive Disclosure

This SKILL.md provides a high-level overview. For detailed instructions:
1. Identify the command you need from the categories above
2. Read the referenced `.md` file for complete workflow details
3. Follow the command's specific steps and guidelines

Commands are designed to be self-contained - each file includes all necessary context for execution.
