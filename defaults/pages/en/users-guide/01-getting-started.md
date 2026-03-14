---
title: Getting Started
order: 1
---

## Getting Started

Create your first documentation site in under a minute.

### Install

```bash
cargo install docs-gen
```

Requires [Rust](https://rustup.rs/). After installation, the `docs-gen` command is available globally.

### 1. Initialize a Project

```bash
docs-gen init my-docs
```

This creates the following structure:

```
my-docs/
├── config.toml              # Site settings
└── pages/
    ├── en/                      # English pages
    │   ├── index.md             # Homepage
    │   ├── users-guide/         # Section (has its own sidebar)
    │   │   ├── index.md
    │   │   └── *.md
    │   └── developers-guide/    # Section for contributors
    │       ├── index.md
    │       └── *.md
    └── ja/                      # Japanese pages (same structure)
```

You can customize the theme later — see [Customizing Themes](../04-customizing-themes/).

### 2. Preview Locally

```bash
docs-gen serve my-docs --open
```

The `--open` flag launches your default browser automatically. A local server starts at `http://localhost:8080` with live-reload enabled.

> Every time you save a Markdown file, `config.toml`, or a template, the browser automatically reloads — no manual refresh needed.

While the server is running, try editing `pages/en/index.md` (or `pages/ja/index.md`) to replace the sample homepage with your own project description. You can also drop your own `favicon.svg` into `static/` to override the default icon.

### 3. Build for Production

```bash
docs-gen build my-docs docs
```

Static HTML is generated in the `docs/` directory, ready to deploy.

> **Tip:** If you `cd my-docs` first, you can omit the directory argument from all commands:
>
> ```bash
> mkdir my-docs && cd my-docs
> docs-gen init                 # same as: docs-gen init my-docs
> docs-gen serve --open         # same as: docs-gen serve my-docs --open
> docs-gen build . docs         # same as: docs-gen build my-docs docs
> ```

Next: [Writing Pages](../02-writing-pages/)
