# Kanban Planner Progress Tracker

## Project Overview
Recursive Kanban Planner (Rust + Dioxus). A recursive, card-based planning system built on the strict abstraction: everything is a card. Compiles to web (WASM), desktop, and mobile from a single Rust codebase.

## Current Progress
- [x] Documented core architecture and clean layered approach into `design_document.md`.
- [x] Established Multi-Agent Architecture and Micro-Step Workflow.
- [x] Greenfielded `src/` — removed legacy models, created `src/domain/` layer.
- [x] Implemented `CardId` and `BucketId` (ULID Newtype wrappers) in `src/domain/id.rs`.
- [x] Switched framework from Leptos to Dioxus for cross-platform support.
- [x] Resolved all open architecture questions (Bucket struct, deletion rules, JSON persistence, Dioxus deployment).

## MVP Remaining Tasks
### 1. Domain Layer (Pure Logic)
- [ ] Implement `Bucket` entity (`BucketId`, `name`).
- [ ] Implement `Card` entity with strict invariants.
- [ ] Implement explicit domain errors (`CardNotFound`, `InvalidParent`, `CycleDetected`, etc.).
- [ ] Implement `CardRegistry` with invariant enforcement (no cycles, valid buckets, single parent).

### 2. Application Layer (Commands & Projections)
- [ ] Create atomic Command handlers (`CreateCard`, `MoveCardToBucket`, `ReparentCard`, `CreateBucket`, etc.).
- [ ] Implement board projection logic (grouping child cards by bucket on demand).

### 3. Infrastructure Layer (Persistence)
- [ ] Define Repository Interface (load, save, fetch card).
- [ ] Implement JSON export/import.
- [ ] Implement browser `localStorage` / `IndexedDB` adapter.

### 4. Interface Layer (Dioxus UI)
- [ ] Set up Dioxus app scaffold with routing (`/`, `/board/:card_id`).
- [ ] Build hierarchical UI navigation (entering a card implies viewing its board).
- [ ] Implement board rendering component (Kanban columns + cards).
- [ ] Connect UI actions to Command handlers.
