---
title: Checking Your Site
order: 5
---

## Checking Your Site

The `check` command validates your documentation source for common issues before you build.

```bash
docs-gen check my-docs
```

### What It Checks

#### Broken Internal Links

Every Markdown link and image reference is resolved against the source pages directory. A broken link (one whose target file does not exist) is reported as an **error**.

> - Relative links like `../sibling/` are resolved from the page's directory
> - Directory links (ending with `/`) expect an `index.md` inside
> - External links (`https://`, `mailto:`, etc.) are skipped
> - Anchor-only links (`#section`) are skipped

#### Duplicate `order` Values

Within each section (subdirectory), every non-index page's `order` frontmatter value is checked. If two or more pages share the same `order`, a **warning** is reported. This helps prevent ambiguous sidebar ordering.

#### Unset `order`

Pages inside a section that leave `order` unset (defaulting to `0`) get a **warning**, since explicit ordering is almost always intended.

#### Unreferenced Pages

Pages that are not linked from any other page's Markdown content get a **warning**. This often indicates a missing link in a table of contents or a "Next" footer.

> These pages are still accessible via sidebar navigation. Index pages (`index.md`) are excluded from this check.

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No errors found (warnings may still be printed) |
| `1` | One or more errors found |
| `2` | Runtime failure (e.g. missing config, unreadable files) |

### Example Output

```
[warn]  [en] guide/: duplicate order 3 in: guide/02-foo.md, guide/03-bar.md
[error] [en] guide/02-foo.md: broken link target: ../nonexistent/
[warn]  [en] guide/04-baz.md: order is not set (defaults to 0)
[warn]  [en] guide/05-orphan.md: page is not referenced by any link

1 error(s), 3 warning(s) found.
```

Next: [Deploying](../06-deploying/)
