---
title: Configuration
order: 3
---

## Configuration

All site settings live in `config.toml` at the project root.

### Basic Structure

```toml
[system]
theme = "default"
langs = ["en", "ja"]

[site]
title = "My Project"
version = "1.0.0"
hostname = "https://example.github.io"
base_path = "/my-project"
footer_message = "© 2026 My Project."
```

### Deployment Scenarios

The `hostname` and `base_path` settings control how URLs are generated.

#### Local development

For local development, you don't need to set `hostname`. Just set `base_path` to empty:

```toml
[site]
base_path = ""
```

This is the default — `docs-gen serve` works out of the box.

#### Custom domain

If your site is served from the root of a domain (e.g. `https://docs.example.com/`):

```toml
[site]
hostname = "https://docs.example.com"
base_path = ""
```

#### Subdirectory (GitHub Pages)

If your site is hosted under a repository name (e.g. `https://user.github.io/my-project/`):

```toml
[site]
hostname = "https://user.github.io"
base_path = "/my-project"
```

### Navigation Buttons

Add buttons to the site header with `[[nav]]` entries:

```toml
[[nav]]
label = "Guide"
path = "guide/"          # Internal path (resolved per language)

[[nav]]
label = "GitHub"
url = "https://github.com/your/repo"   # External URL
```

### Multi-language Support

To add languages, list them in `langs`. The first entry is the default:

```toml
[system]
langs = ["en", "ja"]
```

Then create matching page directories under `pages/ja/`.

> A language switcher will appear in the header automatically when multiple languages are configured.

#### Single-language Sites

For a single-language site, set `langs` to just one entry:

```toml
[system]
langs = ["en"]
```

In single-language mode, URLs have no language prefix (e.g. `/guide/` instead of `/en/guide/`), and the language switcher is hidden.

### Reference

#### `[system]`

| Key | Required | Description |
|-----|----------|-------------|
| `theme` | no | Built-in theme name (default: `"default"`) |
| `langs` | yes | Language codes. First entry is the default. |

#### `[site]`

| Key | Required | Description |
|-----|----------|-------------|
| `title` | yes | Site title displayed in the header |
| `version` | no | Version string displayed in the header |
| `hostname` | no | Base hostname (see [Deployment Scenarios](#deployment-scenarios) and [SEO](#seo) below) |
| `base_path` | no | URL path prefix (see [Deployment Scenarios](#deployment-scenarios) above) |
| `footer_message` | no | Footer text |

#### `[[nav]]` — Header buttons

| Key | Required | Description |
|-----|----------|-------------|
| `label` | yes | Button label text |
| `path` | no | Internal section path relative to `<lang>/` |
| `url` | no | External URL (takes precedence over `path`) |
| `icon_svg` | no | Inline SVG icon markup |

### SEO

When `hostname` is set, docs-gen automatically generates SEO metadata:

- **Canonical URLs** — Each page includes a `<link rel="canonical">` tag to prevent duplicate content issues.
- **hreflang tags** — In multi-language sites, each page includes `<link rel="alternate" hreflang="...">` tags for all configured languages plus `x-default`. This tells search engines that `/en/` and `/ja/` pages are language variants of the same content, not duplicates.
- **sitemap.xml** — A sitemap with all page URLs is generated at the site root. For multi-language sites, it includes `xhtml:link` alternates for each language.

These features require no additional configuration beyond setting `hostname`.

Next: [Customizing Themes](../04-customizing-themes/)
