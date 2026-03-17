# Multi-Agent Architecture Proposal

To ensure the Recursive Kanban Planner maintains its extremely high standards for clean architecture, type safety, and human readability, we propose a Multi-Agent Orchestration system. 

Instead of a single AI trying to write, review, and test code simultaneously, development will be managed by an **Orchestrator Agent** that delegates specialized tasks to focused sub-agents.

## Core Flow
1. **User Request** comes to the Orchestrator.
2. Orchestrator delegates to **Planner** to map out the architectural impact.
3. Orchestrator delegates to **Implementer** to write the code.
4. Orchestrator loops through **Tester**, **Reviewer**, and **Readability** agents.
5. If any reviewer rejects the PR, the Orchestrator sends it back to the Implementer with feedback.
6. Once all passes, the Orchestrator merges and notifies the User.

---

## Proposed Agent Roster

### 1. 🧠 The Orchestrator (The Manager)
*   **Role:** The central router and decision-maker.
*   **Responsibilities:** Interprets user intent, selects the right agents to wake up, gathers their output, handles errors or loops, and decides when a task is "Done" or requires human intervention. 

### 2. 🗺️ Planner Agent (The Architect)
*   **Role:** Enforces Domain-Driven Design and overarching structure.
*   **Responsibilities:** Never writes production code. Modifies `design_document.md`. Answers "How should we build this without breaking strict type safety?" and ensures the Dependency Rule is maintained (preventing UI from bleeding into Domain logic).

### 3. 🛠️ Implementer Agent (The Coder)
*   **Role:** Writes the actual Rust/Leptos code.
*   **Responsibilities:** Receives explicit instructions from the Planner/Orchestrator and writes/modifies `src/` files. Focuses purely on making the code compile and function as requested.

### 4. 🧪 Tester Agent (The QA Engineer)
*   **Role:** Verifies correctness and catches regressions.
*   **Responsibilities:** Writes unit tests for new Domain features (e.g., testing that invariants like `CycleDetected` actually trigger). Runs `cargo test` and `cargo clippy`. Rejects code that fails.

### 5. 🦅 Reviewer Agent (The Gatekeeper)
*   **Role:** Enforces Rust idioms and security.
*   **Responsibilities:** Performs code review on the Implementer's work. Checks for memory leaks, improper use of `unwrap()`, unhandled `Result` types, and ensures maximum type safety is utilized.

### 6. 📖 Human Readability Agent (The Editor)
*   **Role:** Enforces the "Cleanliness & Maintainability First" philosophy.
*   **Responsibilities:** Reviews code strictly for developer experience. Focuses on variable names, module boundaries, docstrings (enforcing `# Examples` in Rust docs), and reducing duplication (DRY). Will reject working code if it is "ugly" or "clever" instead of clear.

### 7. 📦 Dependency & Repo Agent (The Janitor)
*   **Role:** Manages the Cargo ecosystem and repository state.
*   **Responsibilities:** Oversees `Cargo.toml`. Investigates external crates (e.g., assessing `sqlx` vs `rusqlite` for bloat). Keeps dependencies up to date, handles Git commits, and formats the codebase (`cargo fmt`).

### 8. 🚀 Modernizer / Refactoring Agent (The Optimizer)
*   **Role:** Continuous improvement.
*   **Responsibilities:** Periodically scans the codebase to find areas where newer Rust features (e.g., `let else` statements) or cleaner leptos patterns could be applied. Operates only when the system is stable.

---

## Next Steps for User
Please review the proposed agents. 
1. Are there any roles you would like to **add**, **remove**, or **combine**?
2. Once approved, we will translate these into physical `.agents/skills/` definitions so the Orchestrator knows exactly how to invoke them.
