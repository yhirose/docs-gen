---
title: Customizing Themes
order: 4
---

## Customizing Themes

To change colors and styles, install the theme into your project and edit the CSS.

### Installing a Theme

A site created with `docs-gen init` uses the built-in theme as-is. To customize it, install the theme explicitly:

```bash
docs-gen theme install default my-docs
```

This copies the CSS and highlight config into your project so you can edit them:

```text
styles/default/
├── config.toml       # Syntax highlighting settings
├── DESIGN.md         # Theme design intent (for humans and AI agents)
└── static/
    └── css/main.css  # All styles (light/dark themes, layout)
```

> **Note:** Without installation, docs-gen uses its built-in theme and you automatically get improvements when you update docs-gen. Once installed, your local copies take precedence — to pick up future updates, re-install the theme and reapply your customizations.

You can delete any file you don't need to customize and docs-gen will fall back to the built-in default. To also customize templates and JavaScript, add `--with-base` (see [Theme Structure and Creation](../07-creating-themes/)).

### Styles

All styles live in `styles/default/static/css/main.css`. The stylesheet supports both dark and light modes using the `[data-theme="light"]` selector. Dark mode is the default.

To change colors, search for CSS custom properties or edit the relevant selectors directly. Key areas include:

- **Header** — `.header`, `.header-nav`
- **Sidebar** — `.sidebar`, `.nav-section`, `.nav-list`
- **Content** — `.content`, `article`
- **Code blocks** — `pre`, `code`
- **Search modal** — `.search-overlay`, `.search-modal`

### Syntax Highlighting

Code block highlighting is configured in `styles/default/config.toml`:

```toml
[highlight]
dark_theme = "base16-eighties.dark"
light_theme = "InspiredGitHub"
```

Available dark themes: `base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`

Available light themes: `base16-ocean.light`, `InspiredGitHub`, `Solarized (light)`

> When both are set, docs-gen renders each code block twice — once for dark, once for light — and CSS toggles visibility based on the active theme.

### Understanding DESIGN.md

Each built-in theme ships with a `DESIGN.md` file alongside its `config.toml` and CSS. After `docs-gen theme install`, you'll find it at `styles/<theme>/DESIGN.md`.

`DESIGN.md` is a plain-text document describing the theme's design intent:

- The theme's visual philosophy and target feel
- What each CSS custom property means semantically
- Layout and typography rules
- Do's and don'ts when modifying the theme
- Which file to touch for which kind of change

It is **not read at build time** — the file has no effect on generated output. It exists to help humans and AI coding agents make changes that stay consistent with the theme's original intent.

**How to use it:**

- Before editing `main.css` yourself, skim `DESIGN.md` to understand which tokens are safe to change and which rules to preserve.
- When asking an AI coding agent (Claude Code, Cursor, etc.) to customize your theme, the agent will read `DESIGN.md` automatically and produce changes aligned with the theme's design.
- If you heavily customize a theme, update `DESIGN.md` to reflect your new choices so future edits stay coherent.

### Switching Themes

To switch to a different built-in theme, just change the `theme` value in `config.toml` (see [Configuration](../03-configuration/)).

To test a theme without editing `config.toml`, use the `--theme` flag:

```bash
docs-gen serve my-docs --theme monotone
```

Next: [Checking Your Site](../05-checking-your-site/)
