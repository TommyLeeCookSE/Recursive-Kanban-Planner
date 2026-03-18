# Rust Guide for Python Developers

This guide is a quick Rosetta Stone for the Rust syntax, patterns, and memory model used throughout Recursive Kanban Planner.

## 1. Type Safety and Newtypes

In Python, type hints are suggestions. A function like `def get(card_id: str)` will happily accept a `bucket_id` string.

In Rust, we use the newtype pattern to make types explicitly incompatible at compile time:

```rust
pub struct CardId(Ulid);
pub struct BucketId(Ulid);
```

Even though both wrap a `Ulid`, passing a `BucketId` into a function that expects a `CardId` is a compile error. That keeps domain logic from accidentally swapping identifiers.

## 2. Enums and Pattern Matching

In Python, you might reach for `if/elif` chains or structural matching. In Rust, enums and `match` are central tools.

Rust enums are sum types, so variants can carry data:

```rust
pub enum Command {
    CreateCard { title: String },
    DeleteCard { id: CardId },
}
```

When you `match` on an enum, the compiler forces you to handle every case.

## 3. The `#[derive(...)]` Macro

You will see lines like this above many structs:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
```

This macro generates boilerplate automatically:

- `Debug`: similar to Python's `__repr__`
- `Clone, Copy`: allow cheap value-style copying for small types
- `PartialEq, Eq`: support equality comparisons
- `Hash`: allows the type to be used as a `HashMap` key
- `Serialize, Deserialize`: generate JSON conversion code through Serde

## 4. Structs and `impl` Blocks

In Python, a class usually owns both data and methods in one block. In Rust, data (`struct`) and behavior (`impl`) are separated:

```rust
pub struct CardId(Ulid);

impl CardId {
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}
```

An associated function without `&self` is closest to a Python `@classmethod` or `@staticmethod`.

## 5. `Result` and Error Handling

Python uses exceptions. Rust usually returns `Result<T, E>`:

```rust
fn create_card() -> Result<CardId, DomainError> { ... }
```

Callers must handle the `Result` or propagate it with `?`.

## 6. Signals and Hooks in Dioxus

The interface layer uses `Signal<T>` and hooks such as `use_signal`.

- Signals are reactive variables. Updating a signal causes dependent UI to re-render.
- `use_context_provider` lets a parent share state with descendants without prop drilling.
- `rsx!` is Dioxus syntax for HTML-like UI written directly in Rust.

## 7. WASM Constraints

Because part of this code runs in the browser, some standard Rust APIs are not always available. The project uses `#[cfg(target_arch = "wasm32")]` blocks to keep platform-specific code explicit.
