# Kanban Planner Progress Tracker

## Project Overview
Recursive Kanban Planner (Rust Implementation). A recursive, card-based planning system designed to unify task management across individuals, teams, households, and organizations into a single coherent model using a strict abstraction: everything is a card.

## Current Progress
- [x] Initialized Cargo project (`Cargo.toml` with `leptos`, `axum`, `tokio`, `serde`, `uuid` dependencies)
- [x] Basic source files created (`main.rs`, `lib.rs`, `app.rs`, `components.rs`, `routes.rs`, `error_templates.rs`)
- [x] Models directory structure created (`models/`)
  - `Card` struct with standard properties (id, title, bucket, parent/children relationships)
  - `CardRegistry` implementation for in-memory card management
- [x] Documented core architecture and clean layered approach into `design_document.md`.

## MVP Remaining Tasks
### 0. Foundation & Architecture Refactoring
- [ ] Enforce strict decoupling: Restructure `src/` into `domain/`, `application/`, `infrastructure/`, and `interface/`.
- [ ] Remove or adapt existing MVP models (`src/models/card.rs`, `src/models/card_registry.rs`) into the new `domain/` layer.

### 1. Domain Layer (Pure Logic)
- [ ] Refactor existing `Card` and `CardRegistry` to strictly match the design spec (e.g., `Option<BucketId>`, separate `Bucket` entity, invariant checks).
- [ ] Implement explicit domain errors (`CardNotFound`, `InvalidParent`, `CycleDetected`, etc.).
- [ ] Implement core invariant validations (no cycles, valid buckets, single parent, etc.).

### 2. Application Layer (Commands & Projections)
- [ ] Create explicit Command handlers (`CreateCard`, `MoveCardToBucket`, `ReparentCard`, `CreateBucket`, etc.).
- [ ] Implement board projection logic (grouping child cards by bucket on demand).

### 3. Infrastructure Layer (Persistence)
- [ ] Define Repository Interface (load, save, fetch card).
- [ ] Implement SQLite Persistence Adapter.
- [ ] Ensure state can be fully reloaded and persisted.

### 4. Interface Layer (Web UI via Leptos)
- [ ] Connect `main.rs` and `lib.rs` for `axum` and `leptos` server functions.
- [ ] Build hierarchical UI navigation (entering a card implies viewing its board).
- [ ] Implement board rendering component (Kanban UI).
- [ ] Connect UI actions to Command handlers.
