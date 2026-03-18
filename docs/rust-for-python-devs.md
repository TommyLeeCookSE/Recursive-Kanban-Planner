# Rust Guide for Python Developers

This guide serves as a Rosetta Stone for understanding the Rust syntax, boilerplate, and memory paradigms used throughout the Recursive Kanban Planner codebase.

## 1. Type Safety and "Newtypes"

In Python, type hints are suggestions. `def get(card_id: str)` will happily accept a `bucket_id` string.
In Rust, we use the **Newtype Pattern** to make types explicitly incompatible.

```rust
pub struct CardId(Ulid);
pub struct BucketId(Ulid);
```

Even though they both just hold a `Ulid` internally, passing a `BucketId` into a function expecting a `CardId` results in a total compilation failure. This guarantees our domain logic never accidentally swaps IDs.

## 2. The `#[derive(...)]` Macro

You will see this line above almost every struct:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
```

This is a macro that auto-generates boilerplate code:

- `Debug`: Implements Python's `__repr__` equivalent, allowing `println!("{:?}", x)`.
- `Clone, Copy`: Tells Rust the data is small enough to copy by value (like an `int` in Python) rather than moving ownership.
- `PartialEq, Eq`: Implements Python's `__eq__`, allowing `if id1 == id2`.
- `Hash`: Allows the struct to be used as a Dictionary (HashMap) key.

## 3. Structs and `impl` Blocks

In Python, you define a `class` and put its methods indented inside it.
In Rust, data (`struct`) and behavior (`impl`) are strictly separated.

```rust
// The Data
pub struct CardId(Ulid);

// The Behavior
impl CardId {
    // A function with NO `&self` parameter is an Associated Function.
    // Equivalent to a Python `@classmethod` or `@staticmethod`.
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}
```

## 4. Result and Error Handling

Python uses `try/except`. Rust uses the `Result<T, E>` enum.

```rust
fn create_card() -> Result<CardId, DomainError> { ... }
```

You cannot ignore a `Result`. You must either handle it with a `match` statement or use the `?` operator (similar to "bubbling up" an exception) to return the error to the caller.

## 5. The `Signal` and `rsx!` (Dioxus Spec)

In the interface layer, we use `Signal<T>`. This is like a reactive "observable" variable. When you update a `Signal`, the UI automatically re-renders relevant parts, similar to modern Python web frameworks like Reflex or Streamlit, but with much higher performance.
