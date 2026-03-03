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
langs = ["en"]

[site]
title = "My Project"
version = "1.0.0"
hostname = "https://example.github.io"
base_path = "/my-project"
footer_message = "© 2026 My Project."
```

### Deployment Scenarios

The `hostname` and `base_path` settings control how URLs are generated. Configure them according to your hosting setup.

#### Root deployment (custom domain or local)

If your site is served from the root of a domain (e.g. `https://docs.example.com/`):

```toml
[site]
hostname = "https://docs.example.com"
base_path = ""
```

> For local development only, you can omit `hostname` entirely and just set `base_path = ""`.

#### Subdirectory deployment (GitHub Pages)

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
| `hostname` | no | Base hostname (see [Deployment Scenarios](#deployment-scenarios) above) |
| `base_path` | no | URL path prefix (see [Deployment Scenarios](#deployment-scenarios) above) |
| `footer_message` | no | Footer text |

#### `[[nav]]` — Header buttons

| Key | Required | Description |
|-----|----------|-------------|
| `label` | yes | Button label text |
| `path` | no | Internal section path relative to `<lang>/` |
| `url` | no | External URL (takes precedence over `path`) |
| `icon_svg` | no | Inline SVG icon markup |

Next: [Customizing Themes](../04-customizing-themes/)
