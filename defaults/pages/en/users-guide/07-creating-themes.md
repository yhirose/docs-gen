---
title: Creating Themes
order: 7
---

## Creating Themes

> This page is for **docs-gen contributors** who want to add a new built-in theme. If you just want to customize the look of your site, see [Customizing Themes](../04-customizing-themes/).

docs-gen discovers themes automatically at compile time from `defaults/themes/`. To add a new built-in theme, create a directory under `defaults/themes/<name>/` with this structure:

```
defaults/themes/<name>/
├── config.toml          # Highlight theme settings
├── templates/
│   ├── base.html
│   ├── page.html
│   └── portal.html
└── static/
    ├── css/main.css
    ├── js/main.js
    └── favicon.svg
```

No code changes are needed — the theme is available immediately after recompilation.

### Theme `config.toml`

Each theme has its own `config.toml` for syntax highlighting settings:

```toml
[highlight]
dark_theme = "base16-eighties.dark"   # Dark mode theme
light_theme = "InspiredGitHub"        # Light mode theme (optional)
```

| Key | Default | Description |
|-----|---------|-------------|
| `dark_theme` | `base16-eighties.dark` | Theme for dark mode |
| `light_theme` | `InspiredGitHub` | Theme for light mode. Both dark and light code blocks are rendered and toggled via CSS. |

For the full list of available highlight themes, see [Customizing Themes — Syntax Highlighting](../04-customizing-themes/#syntax-highlighting).

### Templates

Templates use the [Tera](https://keats.github.io/tera/) template engine. A theme must provide three templates:

- **base.html** — Shared HTML shell (header, footer, scripts). All other templates extend this.
- **page.html** — Content pages with sidebar navigation.
- **portal.html** — Homepage layout (full-width, no sidebar).

See [Customizing Themes](../04-customizing-themes/) for template variables and styling details.

### Static Assets

The `static/` directory contains CSS, JavaScript, and icons. These are copied as-is to the output during build. At minimum, provide:

- `css/main.css` — All styles including dark/light mode support
- `js/main.js` — Search, theme toggle, language switcher
- `favicon.svg` — Site icon
