---
title: Architecture
order: 2
---

## Architecture

docs-gen is a single-binary static site generator. All source lives under `src/` and all embedded assets live under `defaults/`.

### Module Overview

```
src/
├── main.rs        CLI entry point and commands (init, theme)
├── config.rs      config.toml parsing and validation
├── builder.rs     Site generation: page collection, Tera rendering, output
├── markdown.rs    Frontmatter parsing and syntax-highlighted Markdown rendering
├── serve.rs       Development server with live-reload (HTTP + WebSocket)
├── check.rs       Source validation (broken links, order issues, unreferenced pages)
├── defaults.rs    Compile-time embedding of themes and scaffold files
└── utils.rs       Shared helpers (recursive directory copy)
```

### How a Build Works

When `docs-gen build src out` runs, the pipeline is:

1. **Load config** — `config.rs` reads `config.toml` and merges theme config (project override or built-in default).
2. **Set up Tera** — `builder.rs` registers built-in templates, then overlays any project-level templates from `themes/<name>/templates/` and `templates/`.
3. **Collect pages** — For each language, walk `pages/<lang>/`, parse frontmatter and render Markdown to HTML.
4. **Build navigation** — Group pages by section, sort by `order` then filename, and build the sidebar tree.
5. **Render templates** — Each page gets a Tera context (`site`, `page`, `nav`, `content`, etc.) and is rendered through either `portal.html` or `page.html`.
6. **Copy static files** — Built-in theme static files are written first, then project-level overrides are copied on top.
7. **Generate search index** — `pages-data.json` is written with title, URL, and truncated plain-text body for each page.
8. **Root redirect** — For multi-language sites, an `index.html` redirect is generated based on the default language.

### Embedded Defaults

`defaults.rs` uses `include_dir!` to embed the entire `defaults/` directory at compile time:

- **Themes** (`defaults/themes/`) — Templates, CSS, JS, and config for each built-in theme. Adding a new directory here automatically registers a new theme with no code changes.
- **Scaffold files** (`defaults/config.toml`, `defaults/pages/`) — Copied during `docs-gen init` to bootstrap a new project.

### Development Server

`serve.rs` runs three components in parallel:

- **HTTP server** (`tiny_http`) — Serves the built site from a temporary directory. Handles clean URL routing (directory → `index.html`) and trailing-slash redirects.
- **WebSocket server** (`tungstenite`) — Accepts browser connections for live-reload notifications.
- **File watcher** (`notify`) — Watches the source directory recursively. On change, rebuilds the site, injects the live-reload script into HTML files, and sends a `"reload"` message to all connected browsers. Changes are debounced (200ms) to avoid redundant rebuilds.

### Check Pipeline

`check.rs` runs four validation passes on the source without rendering HTML:

1. **Duplicate order** — Detects pages in the same section sharing an `order` value.
2. **Unset order** — Warns when a non-index page defaults to `order: 0`.
3. **Broken internal links** — Resolves every Markdown link relative to the page's rendered URL path and checks that the target `.md` file exists.
4. **Unreferenced pages** — Finds pages not linked from any other page's content.

The exit code distinguishes clean (0), errors found (1), and runtime failure (2) for CI integration.
