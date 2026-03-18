# Recursive Kanban Planner

A high-performance, premium Kanban application built with **Rust** and **Dioxus**. Organize your life and projects with nested, recursive boards that follow the mantra: **"Everything is a Card."**

---

## 🌟 Concept

In the **Recursive Kanban Planner**, every task is more than just a line of text—it's a **Card**.
- **Nested Boards**: Any card can be opened to reveal its own internal Kanban board.
- **Infinite Recursion**: Break down complex projects into infinite sub-levels without losing context.
- **Clean Structure**: High-level boards stay tidy while detailed work lives deep in the tree.

---

## 🛠️ Technology Stack

- **[Rust](https://www.rust-lang.org/)**: For memory safety, performance, and cross-platform builds.
- **[Dioxus 0.7](https://dioxuslabs.com/)**: A modern UI framework for high-speed WebAssembly and Desktop apps.
- **[Tailwind CSS](https://tailwindcss.com/)**: For the premium, "glassmorphism" aesthetic.
- **[ULID](https://github.com/ulid/spec)**: For collision-resistant, sortable identifiers.
- **[Serde](https://serde.rs/)**: For efficient JSON data persistence.

---

## 🚀 Getting Started

### Prerequisites

You must have the **Rust** toolchain and the **Dioxus CLI** installed:

```bash
# Install Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Dioxus CLI
cargo install dioxus-cli
```

### Running Locally

#### 🌐 Web (WASM)
To launch the browser version:
```bash
dx serve --platform web
```

#### 🖥️ Desktop (Native)
To launch the native desktop application:
```bash
dx serve --platform desktop
```

---

## 🏗️ Architecture

The project follows a strict **Clean Architecture** pattern to ensure maintainability:
- **Domain**: Core entities and business invariants (`id`, `bucket`, `card`, `registry`).
- **Application**: The command dispatcher and high-level view models.
- **Infrastructure**: Persistence facades (LocalStorage) and logging.
- **Interface**: The Dioxus UI components and routing.

For a deeper dive, read the full [Design Document](docs/design_document.md).

---

## 💾 Data & Persistence

> [!WARNING]
> Your data is currently stored in your **Browser's LocalStorage**. Clearing your browser cache or switching devices will result in data loss unless you use the **Export** feature.

- **Auto-Save**: The app saves your registry state after every significant change.
- **Export/Import**: Use the buttons in the Top Navigation bar to download your workspace as a `.json` file for backup or transfer.

---

## 🛡️ License
Distributed under the MIT License. See `LICENSE` for more information.
