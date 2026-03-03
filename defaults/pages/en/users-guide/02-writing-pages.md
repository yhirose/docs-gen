---
title: Writing Pages
order: 2
---

## Writing Pages

All content lives under `pages/<lang>/` as Markdown files.

### Frontmatter

Every `.md` file must begin with YAML frontmatter:

```yaml
---
title: "My Page Title"
order: 1
---
```

| Field    | Required | Description                                   |
|----------|----------|-----------------------------------------------|
| `title`  | yes      | Page title (shown in sidebar and browser tab)  |
| `order`  | no       | Sort order within the section (default: `0`)   |
| `status` | no       | Set to `"draft"` to show a DRAFT banner         |

### Sections and Navigation

Subdirectories under a language become **sections** with sidebar navigation:

```
pages/en/
├── index.md              # Homepage (no sidebar)
└── guide/
    ├── index.md          # Section heading
    ├── 01-intro.md       # Sorted by order, then filename
    └── 02-deploy.md
```

- The section's `index.md` title appears as the sidebar heading.
- Other pages in the section are listed beneath it.
- The root `index.md` uses a portal layout (no sidebar).

### Adding a New Page

1. Create a `.md` file (e.g. `pages/en/guide/03-tips.md`).
2. Add frontmatter with `title` and `order`.
3. The page automatically appears in the sidebar.

### Relative Links Between Pages

Each page is rendered into its own directory to produce clean URLs (e.g. `01-getting-started.md` becomes `01-getting-started/index.html`). This affects how you write relative links.

| Linking from                 | Linking to     | Syntax                                   |
|------------------------------|----------------|------------------------------------------|
| A page (e.g. `01-intro.md`) | A sibling page | `../02-deploy/` (go up one level first)  |
| A section `index.md`        | A child page   | `01-intro/` (link directly)              |

**Example — sibling link:**

```markdown
Next: [Writing Pages](../02-writing-pages/)
```

**Example — index to child:**

```markdown
[Getting Started](01-getting-started/)
```

Next: [Configuration](../03-configuration/)
