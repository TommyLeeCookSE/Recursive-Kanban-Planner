# Workspace Git Issues Tracking (OPEN)

## Issue #100: Consolidate Toolbar Button Components (DRY)
**Status:** OPEN
**Priority:** MEDIUM
**Observation:** Buttons in the `BottomBar`, `NavbarLayout`, and `CardItem` actions manually construct RSX with repeated CSS classes (`app-bar-button`, `app-bar-button-icon`, etc.).
**Recommendation:** Create a reusable `BarButton` component in `src/interface/components/visuals/mod.rs` that standardizes label/icon placement and accessibility.

## Issue #101: Consolidate 'Create Card' Command Variants
**Status:** OPEN
**Priority:** LOW
**Observation:** `Command` enum has three separate variants for card creation (`CreateWorkspaceChildCard`, `CreateChildCard`, `CreateCard`).
**Recommendation:** Unify into a single `CreateCard` command with an `Option<CardId>` for the parent and standardized position handling.

## Issue #102: Accurate WASM Timestamps in Diagnostics
**Status:** OPEN
**Priority:** LOW
**Observation:** `infrastructure/logging.rs` returns `0` for `unix_timestamp_secs` when compiled for WASM.
**Recommendation:** Use `js_sys::Date::now()` or the `web-time` crate to provide correct timing for in-memory diagnostics on the web platform.

## Issue #103: Standardize Modal Action Groups
**Status:** OPEN
**Priority:** MEDIUM
**Observation:** Modal footers (Save/Cancel/Delete) are repetitive across `card.rs`, `notes.rs`, and `search.rs`.
**Recommendation:** Extract a `ModalActions` component to manage standardized spacing and "loading/submitting" states consistently across all modals.
