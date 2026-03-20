# Recursive Kanban Planner

Recursive Kanban Planner is a local-first planning app built with Rust and Dioxus. The core idea is simple: every board is a card, and every card can open into its own board.

## Concept

- The workspace is the single top-level card.
- Every other board is a normal child card beneath it.
- Cards can show their immediate children as compact previews.
- Notes and due dates are built in.
- Clean Architecture keeps domain rules, application commands, infrastructure, and UI concerns separated.

## Current MVP

- Workspace card plus nested child cards
- Card create, edit, move, reparent, and delete flows
- Immediate child previews on cards
- Notebook-style notes with titled plain-text pages on each card
- Date-only due dates with overdue card styling
- Browser persistence through `localStorage`
- Export, import, and clear-cache utilities in the top navigation
- Native and web runtime logging
- Dioxus router-based workspace and board views

## Validation

The canonical local verification command is:

```powershell
pwsh ./scripts/test-all.ps1
```

That script runs:

```text
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo test --doc
cargo check --target wasm32-unknown-unknown
cargo check --no-default-features --features desktop
npm run check:css
```

What still requires manual sanity checking:

- `dx serve --platform desktop` runtime launch in this environment
- Browser interaction feel for drag/drop, notes, and navigation flows

## Getting Started

### Prerequisites

- Install the Rust toolchain with `rustup`
- Install the Dioxus CLI with `cargo install dioxus-cli`
- Add the web target with `rustup target add wasm32-unknown-unknown`
- Install the frontend toolchain with `npm install`

### Run the Web App

```bash
dx serve --platform web
```

To refresh the committed stylesheet after editing `src/interface/tailwind.css`, run:

```bash
npm run build:css
```

If you want live stylesheet rebuilding while you work, run this in a second terminal:

```bash
npm run watch:css
```

To run the browser smoke test and CSS parity check together:

```bash
npm run smoke
```

### Learning Bundle

If you want a single text file to feed into NotebookLM, use:

- `docs/notebooklm_context.txt`

For a guided Rust primer written for Python developers, read:

- `docs/rust-for-python-devs.md`

### Run the Desktop App

```bash
dx serve --platform desktop
```

## GitHub Pages Deployment

This app can be hosted as a static GitHub Pages site using a custom GitHub Actions workflow.

What makes that work:

- The GitHub Pages workflow builds with `--base-path Recursive-Kanban-Planner` so the deployed app loads correctly from the repository subpath.
- The workflow builds the web bundle, then uploads the generated `public/` directory from Dioxus as the Pages site root.
- A copied `404.html` and optional `CNAME` file live inside that `public/` directory so the Dioxus router and custom domains keep working.

To publish it:

1. In the repository settings, set GitHub Pages source to `GitHub Actions`.
2. Push to `main`.
3. Let the workflow in `.github/workflows/github-pages.yml` build and deploy the site.

The Pages workflow now runs the same verification matrix as `./scripts/test-all.ps1` before it builds the deploy artifact.

If GitHub Pages ever starts showing the repository `README.md` instead of the app, the site has usually been switched back to legacy branch publishing (`main` / `/`). Change it back to `GitHub Actions`.

If you rename the repository, update the `--base-path Recursive-Kanban-Planner` value in [.github/workflows/github-pages.yml](.github/workflows/github-pages.yml) to match the new repo slug.

## Persistence

Browser builds save automatically to `localStorage`.

Native builds currently fall back to in-memory state and show a warning banner because a desktop/mobile persistence backend has not been implemented yet.

The top navigation includes working `Export`, `Import`, and `Clear Cache` actions for web builds. Import replaces the active workspace only after the snapshot is validated.

## Architecture

- Domain: entities, identifiers, and invariants
- Application: commands and UI-friendly projections
- Infrastructure: persistence and logging adapters
- Interface: Dioxus components, routes, and modal flows

See [docs/design_document.md](docs/design_document.md) for the detailed architecture contract and [docs/task.md](docs/task.md) for the current execution plan.

## Suggested Next Stages

1. Polish card density, spacing, and the child-preview experience.
2. Add richer search and filtering for larger workspaces.
3. Implement a native persistence backend for desktop/mobile targets.
4. Consider collaboration once the single-user card tree feels stable.

## License

Distributed under the MIT License.
