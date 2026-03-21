# docs-gen

[![CI](https://github.com/yhirose/docs-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/yhirose/docs-gen/actions/workflows/ci.yml)

A zero-dependency static site generator for documentation.
Just write Markdown and deploy — no plugins, no runtime, no extra tools required.

**Documentation:** <https://yhirose.github.io/docs-gen/>

## Main Features

- Built-in themes (dark/light)
- Live-reload development server
- Syntax highlighting
- Client-side full-text search
- Multi-language support

## Installation

```bash
cargo install docs-gen
```

Requires [Rust](https://rustup.rs/). After installation, the `docs-gen` command is available globally.

## Get Started

### 1. Create a project

```bash
docs-gen init my-docs
```

This generates `config.toml` and sample pages:

```ascii
my-docs/
├── config.toml        # Site title, URL, footer, navigation
└── pages/
    ├── en/            # English pages
    │   ├── index.md       # Homepage
    │   ├── users-guide/       # Section (has its own sidebar)
    │   │   ├── index.md
    │   │   ├── *.md           # Pages with frontmatter
    │   │   └── screenshot.png # Images next to pages (colocation)
    │   └── developers-guide/  # Section for contributors
    │       ├── index.md
    │       └── *.md           # Pages with frontmatter
    └── ja/            # Japanese pages (same structure as en/)
```

### 2. Preview locally

```bash
docs-gen serve my-docs --open
```

The `serve` command watches your files and automatically reloads the browser whenever you save a change. Keep it running while you edit — every save is reflected instantly.

### 3. Edit config.toml

Update the settings for your site. The generated file includes comments explaining each field. See the [User's Guide](https://yhirose.github.io/docs-gen/users-guide/) for more.

```toml
[system]
theme = "default"                          # Run `docs-gen theme list` to see options
langs = ["en", "ja"]                       # Language codes (first is default)

[site]
title = "My Docs"
version = "0.1.0"
hostname = "https://username.github.io"    # Your site's hostname
base_path = "/my-project"                  # URL path prefix (or "" if served from root)
footer_message = "© 2026 Your Name. All rights reserved."

[[nav]]
label = "Guide"
path = "users-guide/"
```

### 4. Write content

Add `.md` files anywhere under `pages/en/` — they become pages automatically. Group related pages into subdirectories (each with an `index.md`) to create sections with sidebar navigation. Each page starts with a frontmatter header:

```markdown
---
title: My New Page
order: 2            # Unique within the section, controls sidebar order
---

Your content here...
```

### 5. Check, build, and deploy

```bash
docs-gen check my-docs
docs-gen build my-docs docs
```

The `docs/` directory is plain static HTML — deploy it to any hosting service.

> **GitHub Pages:** commit and push `docs/`, then go to **Settings > Pages**, set **Source** to **Deploy from a branch**, and choose your branch with `/docs` folder. See the [Deploying guide](https://yhirose.github.io/docs-gen/users-guide/06-deploying/) for GitHub Actions, Netlify, and other options.

For more details — image handling, multi-language setup, theme customization, and more — see the [User's Guide](https://yhirose.github.io/docs-gen/users-guide/).

---

## Commands

| Command | Description |
|---------|-------------|
| `init [DIR] [--theme NAME]` | Scaffold a new project (default: current directory) |
| `serve [SRC] [--port PORT] [--open] [--theme NAME]` | Build and serve locally with live-reload |
| `build SRC OUT [--theme NAME]` | Generate static HTML for deployment |
| `check [SRC] [--fix]` | Check for broken links, duplicate orders, and other issues |
| `theme list` | List available built-in themes |
| `theme install NAME [SRC] [--force]` | Install a built-in theme into the project |

---

## Built-in Themes

| Theme | Description |
|-------|-------------|
| `default` | Dark/light theme with color accents |
| `monotone` | Calm, sophisticated grayscale-only theme (dark/light) |

Use `--theme NAME` with `init`, `serve`, or `build` to select a theme. See the [Customizing Themes guide](https://yhirose.github.io/docs-gen/users-guide/04-customizing-themes/) for details.

---

## Learn More

- [Writing Pages](https://yhirose.github.io/docs-gen/users-guide/02-writing-pages/) — Markdown features, frontmatter options, and page organization
- [Configuration](https://yhirose.github.io/docs-gen/users-guide/03-configuration/) — All config.toml settings explained
- [Customizing Themes](https://yhirose.github.io/docs-gen/users-guide/04-customizing-themes/) — Install and edit themes to match your brand
- [Checking Your Site](https://yhirose.github.io/docs-gen/users-guide/05-checking-your-site/) — Validate links and order before building
- [Deploying](https://yhirose.github.io/docs-gen/users-guide/06-deploying/) — GitHub Pages and other hosting services
- [Creating Themes](https://yhirose.github.io/docs-gen/users-guide/07-creating-themes/) — Build your own theme from scratch
- [Developer's Guide](https://yhirose.github.io/docs-gen/developers-guide/) — Contributing and working on docs-gen itself

---

## License

MIT
