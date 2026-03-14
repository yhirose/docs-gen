---
title: Customizing Themes
order: 4
---

## Customizing Themes

docs-gen ships with built-in themes, but you can fully customize the look and feel of your site by installing a theme into your project and editing its files.

### Installing a Theme

Theme files are **not** copied during `init`. To customize the theme, install it explicitly:

```bash
docs-gen theme install default my-docs
```

This copies the theme files into `my-docs/themes/default/` so you can edit them.

> **Note:** You only need to install a theme if you want to customize it. Without installation, docs-gen uses its built-in theme automatically and picks up improvements when you update docs-gen. Once installed, your local copy takes precedence — to get future updates, you'll need to re-install the theme and reapply your modifications manually.

Once installed, the theme directory looks like this:

```
themes/default/
├── config.toml              # Syntax highlighting settings
├── templates/
│   ├── base.html            # Shared layout (header, footer, scripts)
│   ├── page.html            # Content pages with sidebar
│   └── portal.html          # Homepage (full-width, no sidebar)
└── static/
    ├── css/main.css          # All styles (light/dark themes, layout)
    ├── js/main.js            # Search, theme toggle, language switcher
    └── favicon.svg           # Site icon
```

### Templates

Templates use the [Tera](https://keats.github.io/tera/) template engine (similar to Jinja2). There are three templates:

- **base.html** — The shared HTML shell. Contains the header, footer, search modal, and theme-detection script. All other templates extend this.
- **page.html** — Used for content pages. Extends `base.html` and adds a sidebar with section navigation.
- **portal.html** — Used for the homepage (`index.md` at the language root). Extends `base.html` with a full-width layout and no sidebar.

#### Available Template Variables

| Variable | Description |
|----------|-------------|
| `site.title` | Site title from `config.toml` |
| `site.version` | Version string |
| `site.base_url` | Full base URL (`hostname` + `base_path`) |
| `site.base_path` | URL base path |
| `site.footer_message` | Footer HTML |
| `site.nav` | Navigation entries (each has `label`, `path` or `url`) |
| `site.langs` | List of configured languages |
| `site.single_lang` | `true` when only one language is configured |
| `page.title` | Current page title |
| `page.status` | Page status (e.g. `"draft"`) |
| `lang` | Current language code |
| `lang_prefix` | Language path prefix (e.g. `"/en"`), empty for single-language sites |
| `content` | Rendered page HTML. Use with `\| safe` to output raw HTML without escaping. |
| `nav` | Sidebar navigation tree (in `page.html`) |

### Styles

All styles live in `static/css/main.css`. The stylesheet supports both dark and light modes using the `[data-theme="light"]` selector. Dark mode is the default.

To change colors, search for CSS custom properties or edit the relevant selectors directly. Key areas include:

- **Header** — `.header`, `.header-nav`
- **Sidebar** — `.sidebar`, `.nav-section`, `.nav-list`
- **Content** — `.content`, `article`
- **Code blocks** — `pre`, `code`
- **Search modal** — `.search-overlay`, `.search-modal`

### Syntax Highlighting

Code block highlighting is configured in the theme's `config.toml`:

```toml
[highlight]
dark_theme = "base16-eighties.dark"
light_theme = "InspiredGitHub"
```

Available dark themes: `base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`

Available light themes: `base16-ocean.light`, `InspiredGitHub`, `Solarized (light)`

> When both are set, docs-gen renders two versions of each code block and toggles them with CSS based on the active theme.

### JavaScript

`static/js/main.js` handles interactive features:

- **Theme toggle** — Switches between dark and light mode, saving the preference to `localStorage`
- **Language switcher** — Appears automatically when multiple languages are configured
- **Search** — Client-side full-text search triggered by the search button or `⌘K` / `Ctrl+K`

### Switching Themes

To use a different built-in theme, change the `theme` value in your project's `config.toml` (see [Configuration](../03-configuration/)) and install it:

```bash
docs-gen theme install monotone my-docs
```

To test a theme without editing `config.toml`, use the `--theme` flag:

```bash
docs-gen serve my-docs --theme monotone
```

Next: [Checking Your Site](../05-checking-your-site/)
