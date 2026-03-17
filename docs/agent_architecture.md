# Multi-Agent Architecture

The Recursive Kanban Planner is developed using a structured multi-agent system. Each agent has a narrow role, reads its own `SKILL.md` for instructions, and communicates back through the Orchestrator.

**All agent skill definitions live in `.agents/skills/`.**

---

## Core Flow

1. **User Request** → Orchestrator interprets intent.
2. Orchestrator reads the required agent skill files and activates the appropriate agents.
3. **Planner** designs the architecture or confirms the spec before any code is written.
4. **Implementer** writes production `src/` code per the Planner's spec.
5. **Tester** writes tests and runs the full verification suite.
6. **Reviewer** and **Readability** agents gatekeep code quality.
7. If any agent rejects the work, the Orchestrator routes feedback back to the Implementer.
8. Once all passes, the Orchestrator commits and notifies the User.

---

## Agent Roster

### 🧠 Orchestrator (The Manager)
- Central router and decision-maker.
- Reads skill files, delegates to agents, manages feedback loops.
- Skill: `.agents/skills/orchestrator/SKILL.md`

### 🗺️ Planner (The Architect)
- Enforces Domain-Driven Design and layered architecture.
- Never writes production code. Maintains `docs/design_document.md`.
- Skill: `.agents/skills/planner/SKILL.md`

### 🛠️ Implementer (The Coder)
- Writes all Rust and Dioxus code in `src/`.
- Receives explicit specs from Planner/Orchestrator. Does not invent architecture.
- Skill: `.agents/skills/implementer/SKILL.md`

### 🧪 Tester (The QA Engineer)
- Writes unit and integration tests. Runs the full verification suite.
- Rejects implementations that fail `cargo test`, `cargo clippy`, or `cargo fmt`.
- Skill: `.agents/skills/tester/SKILL.md`

### 🦅 Reviewer (The Gatekeeper)
- Enforces idiomatic Rust: no `.unwrap()` in production, no unhandled `Result`, no type erasure.
- Skill: `.agents/skills/reviewer/SKILL.md`

### 📖 Readability (The Editor)
- Enforces documentation and naming standards.
- Requires `/// # Examples` on all public domain items.
- Skill: `.agents/skills/readability/SKILL.md`

### 📦 Janitor (The Repo Manager)
- Manages `Cargo.toml`, `cargo fmt`, conventional commits, and dependency hygiene.
- Skill: `.agents/skills/janitor/SKILL.md`

### 🚀 Optimizer (The Modernizer)
- Applies modern Rust syntax and removes performance anti-patterns.
- Operates only when the codebase is stable and all tests pass.
- Skill: `.agents/skills/optimizer/SKILL.md`

---

## Technology Stack
- **Language:** Rust (edition 2024)
- **UI Framework:** Dioxus 0.6 (WASM + desktop + mobile from one codebase)
- **Identifiers:** ULID via the `ulid` crate
- **Error types:** `thiserror`
- **Serialization:** `serde` + `serde_json` (infrastructure layer only)
- **No server:** No Axum, no Tokio, no Leptos. Fully client-side.
