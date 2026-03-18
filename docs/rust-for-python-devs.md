# Rust Guide for Python Developers

This guide serves as a Rosetta Stone for understanding the Rust syntax, patterns, and memory paradigms used throughout the Recursive Kanban Planner codebase.

## 1. Type Safety and "Newtypes"

In Python, type hints are suggestions. `def get(card_id: str)` will happily accept a `bucket_id` string.
In Rust, we use the **Newtype Pattern** to make types explicitly incompatible at compile time.

```rust
pub struct CardId(Ulid);
pub struct BucketId(Ulid);
```

Even though they both just hold a `Ulid` internally, passing a `BucketId` into a function expecting a `CardId` results in a total compilation failure. This guarantees our domain logic never accidentally swaps IDs.

## 2. Enums and Pattern Matching (The "Match" Statement)

In Python, you might use `if/elif` or a `switch` statement. In Rust, we use `enum` and `match`.
Rust enums are "Sum Types" — they can actually **hold data**.

```rust
pub enum Command {
    CreateCard { title: String },
    DeleteCard { id: CardId },
}
```

When you `match` on an enum, the compiler **forces** you to handle every single case. If you add a new command and forget to update the handler, the code will not compile.

## 3. The `#[derive(...)]` Macro

You will see this line above almost every struct:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
```

This is a macro that auto-generates boilerplate code:

- `Debug`: Implements Python's `__repr__` equivalent, allowing `println!("{:?}", x)`.
- `Clone, Copy`: Tells Rust the data is small enough to copy by value (like an `int` in Python) rather than moving ownership.
- `PartialEq, Eq`: Implements Python's `__eq__`, allowing `if id1 == id2`.
- `Hash`: Allows the struct to be used as a Dictionary (`HashMap`) key.
- `Serialize, Deserialize`: Provided by `Serde`. This auto-generates code to turn the struct into JSON and back.

## 4. Structs and `impl` Blocks

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

## 5. Result and Error Handling

Python uses `try/except`. Rust uses the `Result<T, E>` enum.

```rust
fn create_card() -> Result<CardId, DomainError> { ... }
```

You cannot ignore a `Result`. You must either handle it with a `match` statement or use the `?` operator to "bubble up" the error to the caller. This is much safer than Python's exceptions because you can see exactly which functions might fail.

## 6. Signals and Hooks (Dioxus Spec)

In the interface layer, we use `Signal<T>` and hooks like `use_signal`.

- **Signals**: Think of these as "reactive variables". When you update its value (`signal.set(new_value)`), any part of the UI that reads that signal (`signal()`) automatically re-renders. This is similar to State in React or "Observables" in some Python frameworks.
- **`use_context_provider`**: This allows a parent component to "broadcast" a piece of data (like the entire Card Registry) to all of its children, grandchildren, etc., without passing it through every single function call (avoiding "Prop Drilling").
- **`rsx!`**: This is a macro for writing HTML-like structures directly in Rust. It looks like HTML but is strictly typed and extremely fast.

## 7. WASM (WebAssembly) Constraints

Because this code runs in the browser, some standard Rust libraries won't work. For example, `std::time::SystemTime::now()` will crash (panic) in WASM. We use feature-gated code (`#[cfg(target_arch = "wasm32")]`) to handle these differences, much like you might use `sys.platform == "win32"` in Python to handle OS-specific code.
