# Recursive Kanban Planner

Recursive Kanban Planner is a local-first Kanban application built with Rust and Dioxus. The core idea is simple: every task is a card, and every card can open into its own board.

## Concept

- Nested boards let any card become a planning surface for its own child cards.
- Recursive structure keeps high-level boards clean while detailed work lives deeper in the tree.
- Clean Architecture keeps domain rules, application commands, infrastructure, and UI concerns separated.

## Current MVP

- Root boards and nested child cards
- Per-card bucket management
- Card create, rename, move, reparent, and delete flows
- Bucket create, rename, remove, and reorder flows in the domain/application layers
- Browser persistence through `localStorage`
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
- Export/import workflow from the top navigation

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

The `Export` and `Import` buttons are present in the navigation, but their workflows are still planned rather than implemented.

## Architecture

- Domain: entities, identifiers, and invariants
- Application: commands and UI-friendly projections
- Infrastructure: persistence and logging adapters
- Interface: Dioxus components, routes, and modal flows

See [docs/design_document.md](docs/design_document.md) for the detailed architecture contract and [docs/task.md](docs/task.md) for the current execution plan.

## Suggested Next Stages

1. Replace the move dropdown with drag-and-drop for cards and buckets.
2. Add notebook-style notes to cards, including titled note pages.
3. Add due dates and board-level due-state surfacing.
4. Add configurable tags plus event hooks such as note-open, note-close, and bucket-entry automation.

## License

Distributed under the MIT License.
