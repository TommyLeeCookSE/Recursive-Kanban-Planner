# Rust for Python Developers

This guide is a short, practical map from Python ideas to the Rust patterns used in this repository.
It is written to help you read the codebase with less friction, not to teach every Rust feature at once.

## How To Read The App

If you are trying to understand the project, read these files in this order:

1. `README.md`
2. `docs/design_document.md`
3. `src/domain/mod.rs`
4. `src/domain/id.rs`
5. `src/domain/card.rs`
6. `src/domain/registry.rs`
7. `src/application/mod.rs`
8. `src/interface/app.rs`
9. `src/interface/routes/home.rs`
10. `src/interface/routes/board.rs`

That path shows the layers from domain rules to user interface.

## Python To Rust Mental Model

| Python idea | Rust idea in this repo |
| --- | --- |
| Plain object | `struct` with private fields and methods |
| Enum-like tag | `enum` with `match` |
| Runtime exceptions | `Result<T, DomainError>` |
| Optional value | `Option<T>` |
| Module/package | `mod` and `pub mod` |
| Decorators / dataclass helpers | `#[derive(...)]` |
| Type aliases that matter | Newtypes like `CardId`, `BucketId` |
| GUI callbacks | Dioxus event handlers |

## Structs And Impl Blocks

Rust uses `struct` for data and `impl` for behavior.

```rust
pub struct Card {
    id: CardId,
    title: String,
}

impl Card {
    pub fn title(&self) -> &str {
        &self.title
    }
}
```

This is similar to a Python class, but the fields are often private and accessed through methods.
That design helps the domain layer keep invariants inside the type itself.

### Why Private Fields Matter

In this project, `Card`, `Bucket`, and `CardRegistry` protect their own rules.
You usually do not mutate fields directly from the UI.
Instead, you call a method or application command that validates the change first.

## Newtypes

One of the most useful Rust patterns in this codebase is the newtype.

```rust
pub struct CardId(Ulid);
pub struct BucketId(Ulid);
pub struct NotePageId(Ulid);
```

These all wrap the same underlying `Ulid` type, but Rust treats them as distinct types.
That prevents bugs where a function accidentally receives a bucket id when it wanted a card id.

Python can pass the wrong value into a function at runtime.
Rust tries to catch that at compile time.

## Enums And Match

Rust `enum`s are richer than Python enums.
They can hold data, which makes them perfect for commands, states, and UI modes.

```rust
pub enum ModalType {
    CreateCard { parent_id: Option<CardId>, bucket_id: Option<BucketId> },
    EditCard { id: CardId },
}
```

To use an enum, you usually `match` on it:

```rust
match modal {
    ModalType::CreateCard { parent_id, bucket_id } => { /* ... */ }
    ModalType::EditCard { id } => { /* ... */ }
}
```

This is one of the most common patterns in the app shell and modal system.

## Option

`Option<T>` is Rust's "maybe" type.

- `Some(value)` means a value exists
- `None` means it does not

The project uses it for:

- root vs nested cards
- optional due dates
- optional bucket assignment
- optional selected modal state

Example:

```rust
if let Some(due_date) = card.due_date() {
    println!("Due on {}", due_date);
}
```

That is similar to checking `if value is not None` in Python.

## Result And Domain Errors

`Result<T, E>` is the standard Rust success/failure type.

```rust
pub fn new_root(title: String) -> Result<Self, DomainError>
```

This means:

- `Ok(value)` on success
- `Err(error)` on failure

In this repo, domain operations return `DomainError` instead of panicking.
That keeps the UI and persistence layers from silently accepting invalid state.

### Why This Is Better Than Exceptions Here

The app has many user-editable states:

- empty titles
- invalid card reparenting
- duplicate bucket names
- missing children
- invalid persisted JSON

Returning `Result` makes those failure paths explicit.

## Borrowing And References

Rust uses references instead of copying everything around.

```rust
pub fn title(&self) -> &str {
    &self.title
}
```

The `&self` means "borrow this value, do not take ownership".

You will see three common borrowing styles:

- `&T` for an immutable borrow
- `&mut T` for a mutable borrow
- ownership transfer when a value is moved into a function

In this codebase, the application layer often takes `&mut CardRegistry` because commands mutate shared workspace state.

## Ownership In One Sentence

Rust tracks who owns data so it can prevent use-after-free and accidental shared mutation.

For a Python developer, the easiest way to think about ownership is:

- values are moved unless you borrow them
- borrowed values must stay valid
- only one mutable writer is allowed at a time

That is why many functions in the app are carefully split into read-only and mutating forms.

## Derive Macros

You will see a lot of `#[derive(...)]`.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardId(Ulid);
```

This is similar to asking Python to automatically generate common methods for a dataclass.

Common derives in this project:

- `Debug` for logging
- `Clone` for copying values when needed
- `PartialEq` and `Eq` for comparisons
- `Serialize` and `Deserialize` for persistence

## Modules And Visibility

Rust uses modules to organize code and `pub` to expose items outside the module.

```rust
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;
```

Each layer in the project lives in its own module tree.
That keeps domain logic away from UI rendering and persistence details.

### Public Versus Private

Most fields in the domain are private.
That is intentional.

The repo prefers:

- public constructors
- public methods
- private fields
- domain validation in one place

That pattern makes the code easier to reason about and harder to misuse.

## Serde And Persistence

Serde is the serialization framework used here.

- `Serialize` turns Rust data into JSON
- `Deserialize` turns JSON back into Rust data

This repository uses serde to store the workspace in browser `localStorage`.
That means the same domain types are used both in memory and on disk.

Important pattern:

- deserialize data
- validate the structure
- reject corrupted snapshots before the UI uses them

That is one of the main reasons the app stays reliable even when the browser data is stale or edited externally.

## Dioxus Components

Dioxus components are Rust functions that return UI.

```rust
#[component]
pub fn Modal(on_close: EventHandler<()>, title: String, children: Element) -> Element {
    rsx! {
        div { /* ... */ }
    }
}
```

This feels a bit like React components in Python or JavaScript:

- props are function parameters
- UI is declared with `rsx!`
- events are handled with closures

The important difference is that the component body is plain Rust code.

## Signals And Reactive State

The UI uses Dioxus signals to hold shared reactive state.

```rust
Signal<CardRegistry>
Signal<Option<String>>
Signal<Vec<PopupNotification>>
```

Think of these as reactive containers.
When the value changes, Dioxus re-renders the affected parts of the UI.

This is how the board, modals, warnings, and drag state stay synchronized.

## cfg And Target-Specific Code

Rust can compile different code for different targets.

```rust
#[cfg(target_arch = "wasm32")]
```

This app uses that pattern for:

- browser persistence
- browser logging
- web-only drag/drop details
- native fallback behavior

That lets one codebase support both browser and desktop targets.

## Error Handling Philosophy In This Repo

The codebase strongly prefers explicit failure over hidden failure.

When a command fails, the app usually:

1. logs the error
2. records a diagnostic
3. shows a user-friendly warning banner

That is a very Rust-friendly style because it keeps the control flow visible.

## How To Trace A Feature

When you are trying to understand a feature, trace it like this:

1. Start in the UI route or component.
2. Find the command or event handler.
3. Follow it into `src/interface/actions.rs`.
4. Follow the command into `src/application/mod.rs`.
5. Follow the mutation into `src/domain/`.
6. Check persistence in `src/infrastructure/repository.rs`.

That path is the fastest way to understand how a click becomes a state change.

## Short Glossary

- `Card` - the primary recursive planning item
- `Bucket` - a column inside a card's board
- `CardRegistry` - the workspace-wide state container
- `Command` - a mutation request handled by the application layer
- `ModalType` - UI state describing which modal is open
- `DomainError` - typed validation or invariant failure

## Suggested Practice

If you want to practice Rust with this repo, try reading these next:

- `src/domain/id.rs`
- `src/domain/card.rs`
- `src/domain/registry.rs`
- `src/application/mod.rs`
- `src/interface/actions.rs`
- `src/interface/components/modal.rs`

Then compare them with the matching UI routes in `src/interface/routes/`.
