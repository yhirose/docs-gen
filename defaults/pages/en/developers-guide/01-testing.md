---
title: Development Tools
order: 1
---

## Development Tools

docs-gen development uses the [just](https://github.com/casey/just) command runner. Run `just` to see the list of available commands.

### Prerequisites

- **Rust** — Install via `rustup`
- **just** — `cargo install just` or `brew install just`
- **Firefox + geckodriver** — For E2E tests. `brew install geckodriver`

### Commands

#### just install

Install docs-gen locally:

```bash
just install    # cargo install --path .
```

Other commands (`just build`, `just serve`, etc.) use the `docs-gen` binary, so run this first after making changes.

#### just test

Run unit tests and E2E tests together:

```bash
just test
```

This runs the following in sequence:

1. `cargo test --release` — Unit tests
2. `cargo test --release --test e2e -- --ignored --test-threads=1` — E2E tests

E2E tests use [thirtyfour](https://crates.io/crates/thirtyfour) (Rust WebDriver client) with Firefox headless. The test harness automatically starts geckodriver and `docs-gen serve` on ports 4444 and 8123 respectively. Tests run single-threaded to avoid port conflicts.

#### just build

Build the docs-gen documentation site (`docs-src/`):

```bash
just build              # Build with default theme
just build monotone     # Specify a theme
```

#### just serve

Start the documentation site with live-reload and open the browser:

```bash
just serve              # Start with default theme
just serve monotone     # Specify a theme
```

#### just check

Validate the documentation site source (broken links, order issues, etc.):

```bash
just check
```

#### just clean

Remove build output (`docs*/`):

```bash
just clean
```

#### just release

Release a new version:

```bash
just release 0.4.0
```

The release script (`scripts/release`) performs the following:

1. Validates the version format and checks for uncommitted changes
2. Verifies that the latest CI on main has passed
3. Ensures the new version is greater than the current one
4. Updates the version in `Cargo.toml` and `defaults/config.toml`
5. Commits, tags, and pushes

After push, the Publish workflow (`publish.yml`) runs CI again and, if it passes, publishes to crates.io and creates a GitHub Release.
