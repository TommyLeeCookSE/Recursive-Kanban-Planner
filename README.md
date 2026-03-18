# Recursive Kanban Planner

Recursive Kanban Planner is a local-first Kanban application built with Rust and Dioxus. The core idea is simple: every task is a card, and every card can open into its own board.

## Concept

- Nested boards let any card become a planning surface for its own child cards.
- Recursive structure keeps high-level boards clean while detailed work lives deeper in the tree.
- Clean Architecture keeps domain rules, application commands, infrastructure, and UI concerns separated.

## Current MVP

- Root boards and nested child cards
- Per-card bucket management
- Card create, edit, move, reparent, and delete flows
- Bucket create, rename, remove, reorder, and drag-and-drop flows
- Drag-and-drop for cards, buckets, and root boards
- Notebook-style notes with titled plain-text pages on each card
- Date-only due dates with overdue card styling
- Reusable visual labels and popup-only rule presets
- Browser persistence through `localStorage`
- Export, import, and clear-cache utilities in the top navigation
- Native and web runtime logging
- Dioxus router-based workspace and board views

## Verified Status

Validated in this worktree on 2026-03-18:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo check --target wasm32-unknown-unknown
cargo check --no-default-features --features desktop
```

What is not fully verified yet:

- Manual `dx serve --platform desktop` runtime launch in this environment
- Manual browser sanity pass for the newer notebook, due-date, label, rule, and drag/drop flows

## Getting Started

### Prerequisites

- Install the Rust toolchain with `rustup`
- Install the Dioxus CLI with `cargo install dioxus-cli`
- Add the web target with `rustup target add wasm32-unknown-unknown`

### Run the Web App

```bash
dx serve --platform web
```

### Run the Desktop App

```bash
dx serve --platform desktop
```

## Persistence

Browser builds save automatically to `localStorage`.

Native builds currently fall back to in-memory state and show a warning banner because a desktop/mobile persistence backend has not been implemented yet.

The top navigation now includes working `Export`, `Import`, and `Clear Cache` actions for web builds. Import replaces the active workspace only after the snapshot is validated.

## Architecture

- Domain: entities, identifiers, and invariants
- Application: commands and UI-friendly projections
- Infrastructure: persistence and logging adapters
- Interface: Dioxus components, routes, and modal flows

See [docs/design_document.md](docs/design_document.md) for the detailed architecture contract and [docs/task.md](docs/task.md) for the current execution plan.

## Suggested Next Stages

1. Polish the new notebook, due-date, label, and rule UI so dense cards stay easy to scan.
2. Expand rule actions beyond popups once the event model feels stable.
3. Add richer label and rule management ergonomics, including search/filtering for larger workspaces.
4. Implement a native persistence backend for desktop/mobile targets.

## License

Distributed under the MIT License.
