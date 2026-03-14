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

### Images and Files (Colocation)

You can place images and other files in the same directory as your Markdown pages. They are automatically copied to the correct output location during build.

```
pages/en/guide/
├── index.md
├── 01-intro.md
├── diagram.png        ← colocated with this section
└── screenshot.jpg
```

**Referencing from a section `index.md`:**

```markdown
![Diagram](./diagram.png)
```

**Referencing from a regular page** (e.g. `01-intro.md`):

Since each page renders into its own subdirectory (`01-intro/index.html`), use `../` to go up one level:

```markdown
![Screenshot](../screenshot.jpg)
```

**When to use `static/` instead:** Use the `static/` directory for files shared across many pages (e.g. a site logo). Use colocation for files that belong to a specific page or section.

### Image Sizing

By default, images scale to fit the content area (`max-width: 100%`). To control the display size, append a `#fragment` to the image URL:

```markdown
![Screenshot](./screenshot.png#small)
![Screenshot](./screenshot.png#half)
![Screenshot](./screenshot.png#large)
```

| Fragment | Max width |
|----------|-----------|
| *(none)* | 100%      |
| `#small` | 300px     |
| `#half`  | 50%       |
| `#large` | 80%       |

To center an image, add `-center` to the fragment:

```markdown
![Screenshot](./screenshot.png#half-center)
```

You can also use `#center` on its own for a full-width centered image.

### Multi-language Image Fallback

In a multi-language site, images placed in the default language directory (the first entry in `langs`) are automatically available to all other languages. You only need to add a language-specific image if it differs from the default (e.g. a localized screenshot).

```text
pages/
├── en/
│   └── guide/
│       ├── index.md
│       └── screenshot.png   ← shared with all languages
└── ja/
    └── guide/
        └── index.md         ← can reference screenshot.png without having a copy
```

If a language has its own version of the same file, that version takes priority.

Next: [Configuration](../03-configuration/)
