---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 6
research_type: 'technical'
research_topic: 'development-skills-and-tools'
research_goals: 'Inventory available skills, assess value for peach-web, recommend improvements'
user_name: 'Michael'
date: '2026-03-27'
web_research_enabled: true
source_verification: true
---

# Research Report: Development Skills and Tools for peach-web

**Date:** 2026-03-27
**Author:** Michael
**Research Type:** Technical

---

## Research Overview

This report inventories all available development-phase skills and external tools for the peach-web project, assesses their value for a Rust/Leptos CSR + WASM codebase, and recommends actionable improvements to the development workflow.

**Methodology:** Explored installed BMAD skills (134+ total), researched external MCP servers and Claude Code ecosystem tools via web search, and evaluated each against the project's specific tech stack (Cargo workspace, Leptos 0.8, Trunk, WASM).

---

## 1. Installed Development-Phase Skills

### 1.1 Implementation Skills

| Skill | What It Does | When to Use |
|---|---|---|
| `/bmad-dev-story` | Execute story implementation from a spec file | Primary workflow for planned features |
| `/bmad-quick-dev` | Implement small changes from a quick tech spec | Small features, quick changes |
| `/bmad-quick-dev-new-preview` | Implement any intent, requirement, or change request | General-purpose implementation |
| `/bmad-wds-bugfixing` | 5-step structured bug investigation and fix | Tricky bugs that resist quick fixes |
| `/bmad-quick-spec` | Create implementation-ready quick specs | Spec out small changes before coding |

### 1.2 Code Review Skills

| Skill | What It Does | When to Use |
|---|---|---|
| `/bmad-code-review` | Multi-layer adversarial review (Blind Hunter, Edge Case Hunter, Acceptance Auditor) | Post-implementation review |
| `/bmad-review-edge-case-hunter` | Walk every branching path and boundary condition | Complement to code review |
| `/bmad-review-adversarial-general` | Cynical review producing a findings report | Architecture or design review |

### 1.3 Testing Skills

| Skill | What It Does | When to Use |
|---|---|---|
| `/bmad-testarch-framework` | Initialize test framework (Playwright/Cypress) | One-time setup |
| `/bmad-testarch-test-design` | Create system-level or epic-level test plans | Planning test strategy |
| `/bmad-testarch-atdd` | Generate failing acceptance tests (TDD cycle) | Story-level acceptance testing |
| `/bmad-testarch-automate` | Expand test automation coverage | Increasing coverage |
| `/bmad-testarch-ci` | Scaffold CI/CD quality pipeline | CI setup or improvement |
| `/bmad-testarch-test-review` | Review test quality against best practices | Audit existing tests |
| `/bmad-qa-generate-e2e-tests` | Generate e2e automated tests for existing features | Adding e2e tests retroactively |

### 1.4 Built-in Claude Code Skills

| Skill | What It Does | When to Use |
|---|---|---|
| `/simplify` | Review recently changed code for reuse, quality, and efficiency; fix issues found | Post-coding cleanup pass |

---

## 2. External Tools Evaluated

### 2.1 Recommended: None at This Time

After evaluation, no external MCP servers provide sufficient value over existing tooling to justify the setup overhead.

### 2.2 Evaluated and Declined

| Tool | What It Is | Why Declined |
|---|---|---|
| **cargo-mcp** | MCP server wrapping cargo commands | Thin wrapper with no structured error parsing. Claude Code's Bash tool handles cargo output natively. No advantage. |
| **Context7** | MCP server fetching real-time library docs | Partial coverage: Leptos and wasm-bindgen indexed, but web-sys and gloo missing. Proprietary backend, rate-limited free tier. Marginal value given WebFetch access to docs.rs and book.leptos.dev. |
| **Playwright MCP** (Microsoft) | Browser automation via accessibility tree | Useful for e2e testing, but manual browser testing is sufficient for current project phase. Revisit when e2e automation becomes a priority. |
| **Chrome DevTools MCP** (Google) | Live browser inspection via CDP | Same reasoning as Playwright — revisit later. |
| **Sequential Thinking MCP** | Structured step-by-step reasoning | Niche use case. Plan mode and `/bmad-wds-bugfixing` cover the structured reasoning need. |
| **rust-mcp-server / rust-mcp** | Alternative Rust MCP servers | cargo-mcp is more complete, and cargo-mcp itself was declined. |
| **GitHub MCP** | PR/issue management tools | Claude Code's built-in git + gh integration is sufficient. |

### 2.3 Worth Revisiting Later

- **Playwright MCP**: When the project needs automated e2e testing of the WASM UI
- **Context7**: If web-sys and gloo get indexed, or if Leptos API hallucination becomes a recurring problem

---

## 3. Workflow Improvements Implemented

### 3.1 Claude Code Pre-Commit Hook

**Problem:** Despite a cargo-husky pre-commit hook and CI enforcement, `cargo fmt` failures have caused CI rejections multiple times. The pre-commit hook only checks (fails on unformatted code) but doesn't auto-fix.

**Solution:** Added a Claude Code `PreToolUse` hook in `.claude/settings.json` that automatically runs `cargo fmt --all` and re-stages files before every `git commit` command. This ensures all code committed through Claude Code is properly formatted.

### 3.2 CLAUDE.md Skill Reminders

Added a "Development Skills" section to CLAUDE.md documenting when to use key skills:
- `/simplify` — after completing code changes
- `/bmad-code-review` — after implementing stories or significant changes
- `/bmad-wds-bugfixing` — for bugs that resist quick diagnosis

---

## 4. Key Findings

### What's Already Well-Covered

- **Story-driven development**: `/bmad-dev-story` + sprint planning + story creation
- **Test architecture**: Full TEA suite (design, automation, ATDD, CI, review, traceability)
- **Code review**: Multi-layer adversarial review with structured triage
- **CI/CD**: GitHub Actions with quality gate (fmt + clippy)
- **Pre-commit safety**: cargo-husky hook for format/lint checking

### Gaps Identified

1. **Under-used skills**: `/simplify`, `/bmad-code-review`, and `/bmad-wds-bugfixing` were installed but not referenced in CLAUDE.md, making them easy to forget. Now documented.
2. **No Claude Code hooks**: Pre-commit automation relied entirely on cargo-husky (check-only). Now supplemented with auto-formatting hook.
3. **No browser automation**: Acceptable for current project phase, but worth revisiting.

### Ecosystem Gap: No Rust-Specific Skills Exist

The Claude Code skills ecosystem (launched October 2025) is heavily weighted toward web/frontend (React, Next.js) and Apple platforms (Swift). No Rust-specific skills exist for concurrency patterns, WASM development, Cargo workspace management, or Rust-idiomatic code review.

Notable general-purpose alternatives:
- **obra/superpowers** — language-agnostic TDD, debugging, and code review workflows
- **Dimillian/Skills** — includes a `swift-concurrency-expert` skill that could serve as a template for a Rust equivalent
- **AlmogBaku/debug-skill** — interactive DAP debugger supporting Rust

A custom Rust concurrency/WASM skill covering `Send`/`Sync`/`Arc`/`Mutex` patterns and Leptos's `!Send` WASM constraints could be built via `/bmad-agent-builder` if these remain recurring pain points.

### Anti-Patterns to Avoid

- **Circular debugging**: Use `/bmad-wds-bugfixing` for tricky bugs instead of ad-hoc fix attempts. The 5-step workflow (reproduce, investigate, fix, verify, document) prevents the "try a fix, see if it works" loop.
- **Skipping post-coding review**: Run `/simplify` after completing changes. It catches reuse opportunities and quality issues that are easy to miss in the moment.

---

## Sources

- BMAD skill files: `.claude/skills/bmad-*/`
- BMAD WDS bugfixing workflow: `_bmad/wds/workflows/5-agentic-development/workflow-bugfixing.md`
- cargo-mcp: github.com/jbr/cargo-mcp (source code reviewed)
- Context7: github.com/upstash/context7 (README reviewed, library coverage verified)
- Playwright MCP: github.com/microsoft/playwright-mcp
- Chrome DevTools MCP: github.com/ChromeDevTools/chrome-devtools-mcp
- Claude Code hooks documentation: code.claude.com/docs/en/hooks.md
- awesome-claude-code: github.com/hesreallyhim/awesome-claude-code
