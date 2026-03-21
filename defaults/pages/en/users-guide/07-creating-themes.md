---
title: Creating a Theme
order: 7
---

## Creating a Theme

You can create your own theme based on an existing one. This chapter walks through the steps first, then explains the internal structure of themes.

### Steps

This walkthrough creates a shared Base (`corporate`) and a custom Style (`my-theme`) built on top of it.

#### 1. Create a Project and Install a Theme

Install the theme you want to build on with `--with-base` to copy all Base and Style files into your project:

```bash
docs-gen init my-theme-project
docs-gen theme install default my-theme-project --with-base
```

Your project will look like this:

```text
# Base: shared layout foundation
bases/standard/
├── templates/
│   ├── base.html              # Shared HTML shell (header, footer, scripts)
│   ├── page.html              # Content pages with sidebar
│   └── portal.html            # Homepage (full-width, no sidebar)
└── static/
    ├── js/main.js             # Search, theme toggle, language switcher
    └── favicon.svg            # Site icon

# Style: theme-specific files
styles/default/
├── config.toml                # Syntax highlighting settings
└── static/
    └── css/main.css           # All styles
```

#### 2. Name Your Theme

Rename the copied `styles/default/` to something like `styles/my-theme/`, and update the `theme` value in your project's `config.toml` to match:

```toml
# my-theme-project/config.toml
[system]
theme = "my-theme"
```

You can also rename the Base. For example, if you rename `bases/standard/` to `bases/corporate/`, update each Style's `config.toml` to match:

```toml
# styles/my-theme/config.toml
[system]
base = "corporate"
```

After renaming:

```text
bases/corporate/
├── templates/
│   ├── base.html
│   ├── page.html
│   └── portal.html
└── static/
    ├── js/main.js
    └── favicon.svg

styles/my-theme/
├── config.toml
└── static/
    └── css/main.css
```

#### 3. Edit the Files

Edit the files copied into your project:

- `styles/my-theme/static/css/main.css` — Change colors, fonts, layout
- `styles/my-theme/config.toml` — Change syntax highlighting settings
- `bases/corporate/templates/*.html` — Change HTML structure
- `bases/corporate/static/js/main.js` — Change dynamic features

#### 4. Test It

`docs-gen serve` detects file changes and auto-reloads:

```bash
docs-gen serve my-theme-project
```

#### 5. Remove Unnecessary Files

Delete any files you didn't change. Deleted files will fall back to the built-in defaults.

### Theme Structure: Base + Style

A docs-gen theme is made of two parts — **Base** and **Style**:

- **Base** — Templates (HTML), JavaScript, and icons — the shared layout foundation
- **Style** — CSS and highlight config — what differs between themes

Multiple themes can share the same Base. For example, `default` and `monotone` differ only in their CSS — they share the same Base. Which Base to use is specified in the Style's `config.toml`:

```toml
# styles/default/config.toml
[system]
base = "standard"    # This Style uses the standard Base
```

### Base

The Base contains templates and JavaScript:

- **Templates** — Define the HTML structure of pages: header, sidebar, footer, etc.
- **JavaScript** — Works together with templates to provide dynamic features like search, theme switching, and language switching

```text
bases/standard/
├── templates/
│   ├── base.html              # Shared HTML shell (header, footer, scripts)
│   ├── page.html              # Content pages with sidebar
│   └── portal.html            # Homepage (full-width, no sidebar)
└── static/
    ├── js/main.js             # Search, theme toggle, language switcher
    └── favicon.svg            # Site icon
```

#### Templates

There are three templates in `bases/standard/templates/` (syntax: [Tera](https://keats.github.io/tera/)):

- **base.html** — Shared HTML shell (header, footer, search modal, theme-detection script). All other templates extend this.
- **page.html** — Content pages. Extends `base.html` and adds a sidebar with section navigation.
- **portal.html** — Homepage (`index.md` at the language root). Extends `base.html` with a full-width layout and no sidebar.

When editing templates, be careful not to change the HTML elements and class names that the JavaScript references.

#### Available Template Variables

Variables passed to templates by docs-gen at build time, referenced as `{{ site.title }}`. Values come from the project's `config.toml` and the build process:

| Variable | Description |
| --- | --- |
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

### Style

Style contains the files that define the theme's visual appearance:

- **CSS** — Colors, fonts, spacing, and all visual styles
- **config.toml** — Specifies which Base to use and configures syntax highlighting

```text
styles/my-theme/
├── config.toml       # [system] base = "standard" + highlight settings
└── static/
    └── css/main.css  # All styles
```

#### Light/Dark Mode

CSS defaults to dark mode. Light mode styles use the `[data-theme="light"]` selector:

```css
/* Dark mode (default) */
:root {
    --bg-color: #1a1a2e;
}

/* Light mode */
[data-theme="light"] {
    --bg-color: #ffffff;
}
```

#### Syntax Highlighting

Code block highlighting is configured in `config.toml`:

```toml
[highlight]
dark_theme = "base16-eighties.dark"
light_theme = "InspiredGitHub"
```

Dark themes: `base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`

Light themes: `base16-ocean.light`, `InspiredGitHub`, `Solarized (light)`

When both are set, docs-gen renders each code block twice — once for dark, once for light — and CSS toggles visibility based on the active theme.
