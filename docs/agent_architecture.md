# Multi-Agent Architecture

The Recursive Kanban Planner uses a structured multi-agent workflow. Each agent has a narrow role, reads its own `SKILL.md`, and reports back through the Orchestrator.

All agent skill definitions live in `.agents/skills/`.

## Core Flow

1. User request -> Orchestrator interprets intent.
2. Orchestrator reads the required skill files and activates the right agents.
3. Planner defines or confirms the architecture before implementation starts.
4. Implementer writes production `src/` code from the approved spec.
5. Tester runs the verification suite and adds tests as needed.
6. Reviewer and Readability gatekeep correctness and maintainability.
7. If work is rejected, the Orchestrator routes feedback back to the Implementer.
8. Once the checks pass, the Orchestrator closes the loop with the user.

## Agent Roster

### Orchestrator

- Central router and decision-maker
- Reads skill files, delegates work, and manages feedback loops
- Skill: `.agents/skills/orchestrator/SKILL.md`

### Planner

- Enforces Domain-Driven Design and layered architecture
- Never writes production code
- Maintains `docs/design_document.md`
- Skill: `.agents/skills/planner/SKILL.md`

### Implementer

- Writes Rust and Dioxus production code in `src/`
- Receives explicit specs from Planner or Orchestrator
- Skill: `.agents/skills/implementer/SKILL.md`

### Tester

- Writes tests and runs the verification suite
- Rejects work that fails `cargo test`, `cargo clippy`, or `cargo fmt`
- Skill: `.agents/skills/tester/SKILL.md`

### Reviewer

- Enforces idiomatic Rust
- Rejects unhandled `Result`s, unsafe domain shortcuts, and type-erasing patterns
- Skill: `.agents/skills/reviewer/SKILL.md`

### Readability

- Enforces naming, documentation, and API clarity
- Requires `/// # Examples` on public domain-facing items
- Skill: `.agents/skills/readability/SKILL.md`

### Janitor

- Manages `Cargo.toml`, formatting, dependency hygiene, and repo-state cleanliness
- Skill: `.agents/skills/janitor/SKILL.md`

### Optimizer

- Applies modern Rust syntax and performance cleanups once the codebase is stable
- Skill: `.agents/skills/optimizer/SKILL.md`

## Technology Stack

- Language: Rust (edition 2024)
- UI framework: Dioxus 0.7
- Targets: WebAssembly, desktop, and mobile from one codebase
- Identifiers: ULID via the `ulid` crate
- Error types: `thiserror`
- Serialization: `serde` + `serde_json` in the infrastructure layer
- Styling: Tailwind CSS
- No server: fully client-side
