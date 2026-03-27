# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**peach-web** is a Rust/Leptos ear-training web app using BMAD v6.0.4 for AI-orchestrated development. The codebase is a Cargo workspace with two crates (`domain/` for pure Rust logic, `web/` for Leptos CSR + browser APIs) built via Trunk to WASM. BMAD agents, workflows, and structured artifact generation drive the planning and implementation process.

User: Michael | Language: English | Skill level: intermediate

## Architecture

### Module Structure

```
_bmad/
├── _config/          # Global manifests (agent, workflow, task, tool, files CSVs)
├── _memory/          # Persistent context sidecars (tech-writer, storyteller)
├── core/             # Framework core: BMad Master agent, party-mode, workflow executor
├── bmm/              # Build Master Module: 11 project team agents + development workflows
├── bmb/              # Builder Module: meta-system for creating new agents/workflows/modules
├── cis/              # Creative Intelligence Suite: brainstorming, design thinking, innovation
└── tea/              # Test Architecture Enterprise: test design, automation, CI/CD, ATDD
```

### Output Structure

All generated artifacts are saved to `docs/`:
- `planning-artifacts/` — PRDs, architecture docs, UX designs, research reports
- `implementation-artifacts/` — Stories, code implementations
- `ios-reference/` — Domain blueprint and reference specs from iOS app
- `project-context.md` — AI agent coding rules and conventions

### Configuration

- Module configs: `_bmad/<module>/config.yaml` (loaded at agent activation, MANDATORY step 2)
- Global manifests: `_bmad/_config/*.csv` and `manifest.yaml`
- Variable resolution: `{project-root}`, `{config_source}`, `{user_name}`, `{output_folder}`, etc.
- Project knowledge base: `docs/`

## Workflow Execution Model

Workflows are executed via `_bmad/core/tasks/workflow.xml`:

1. **Load**: Read workflow.yaml, resolve variables from config, load instructions
2. **Execute**: Process each step in exact numerical order — never skip steps
3. **Output**: At `template-output` tags, save content and wait for user confirmation before proceeding
4. **YOLO mode**: Skips confirmations, auto-simulates expert discussions

Critical rules:
- Always read COMPLETE workflow files (never use offset/limit)
- Steps are MANDATORY and execute in exact order
- Save to output file after every `template-output` tag
- `[a] Advanced Elicitation` is available at template-output checkpoints

## Interacting with BMAD

All BMAD functionality is accessible via slash commands (skills) in `.claude/commands/`. Key workflows by phase:

**Analysis**: `/bmad-bmm-create-product-brief`, `/bmad-bmm-domain-research`, `/bmad-bmm-market-research`
**Planning**: `/bmad-bmm-create-prd`, `/bmad-bmm-create-ux-design`, `/bmad-bmm-create-architecture`, `/bmad-bmm-create-epics-and-stories`
**Implementation**: `/bmad-bmm-dev-story`, `/bmad-bmm-create-story`, `/bmad-bmm-sprint-planning`, `/bmad-bmm-code-review`
**Quality**: `/bmad-tea-testarch-*` commands for test design, automation, CI, ATDD, traceability
**Utilities**: `/bmad-help` (context-aware next steps), `/bmad-party-mode` (multi-agent discussions)

## Development Skills

Use these skills at the appropriate moments during development:

- `/simplify` — Run after completing code changes to review for reuse, quality, and efficiency issues
- `/bmad-code-review` — Run after implementing stories or significant changes for multi-layer adversarial review
- `/bmad-wds-bugfixing` — Use for bugs that resist quick diagnosis; enforces reproduce-first, root-cause-before-fix discipline to prevent circular debugging

## Agent Conventions

- Agents have persona-driven identities with specific roles and communication styles
- Menu-based interfaces with numbered items and fuzzy command matching
- Agents must load module config at activation (step 2, non-negotiable)
- Resources are loaded at runtime, never pre-loaded

## Documentation Standards

- Strict CommonMark specification compliance
- ATX-style headers only (`#`), fenced code blocks with language identifiers
- Active voice, present tense, task-oriented
- NO time estimates unless explicitly requested
- Mermaid diagrams: valid v10+ syntax, 5-10 nodes ideal, max 15
