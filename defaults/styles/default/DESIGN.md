# Default Theme — Design Specification

This document describes the design intent of the `default` theme for
docs-gen. AI coding agents should read this file before modifying
`static/css/main.css` or `config.toml`, so that changes stay consistent
with the theme's character.

## Design Philosophy

The `default` theme is aimed at **technical documentation sites that feel
calm, legible, and faintly vintage**. It favors warmth over sterility: muted
backgrounds, gold and sky-blue accents, and pastel emphasis colors — closer
to a well-worn terminal or a reference book than to a modern SaaS marketing
site.

- **Content first.** Chrome (header, sidebar) should recede; prose,
  headings, and code should dominate.
- **Dark mode is the primary experience.** Light mode is a faithful
  counterpart, not an afterthought, but color decisions start from dark.
- **Readable, not flashy.** No gradients, heavy shadows, webfonts, or
  animation beyond simple hover fades.
- **Tokens over hard-coded values.** Any color or dimension that might
  reasonably change is a CSS custom property on `:root`.

## Color Tokens

All colors are CSS custom properties. Dark mode values live in `:root`;
light mode overrides live in `[data-theme="light"]`. **Every token defined
in one must be defined in the other.**

### Surfaces
| Token            | Role                                              |
| ---------------- | ------------------------------------------------- |
| `--bg`           | Page background. The base canvas.                 |
| `--bg-secondary` | Header, search modal, table header, hover fills. |
| `--bg-code`      | Inline and block code backgrounds.                |

### Text
| Token                | Role                                          |
| -------------------- | --------------------------------------------- |
| `--text`             | Body copy default.                            |
| `--text-bright`      | Highest-contrast text (hovered title, input). |
| `--text-muted`       | Sidebar items, footer, captions, blockquote.  |
| `--text-code`        | Code block foreground.                        |
| `--text-inline-code` | Inline `` `code` `` foreground — an accent.   |

### Accents
| Token                  | Role                                      |
| ---------------------- | ----------------------------------------- |
| `--link`               | Inline link color. Warm gold in dark.     |
| `--heading`            | `h1` — the page's hero color. Sky blue.   |
| `--heading-link`       | `h2` — secondary heading, warm tan.       |
| `--emphasis`           | `strong`, active nav item. Pink/rose.     |
| `--header-nav-link`    | Top-navigation links.                     |
| `--nav-section`        | Sidebar section label (inactive).         |
| `--nav-section-active` | Sidebar section label (current page).     |

### Borders
| Token           | Role                                |
| --------------- | ----------------------------------- |
| `--border`      | Default separator (header, modal).  |
| `--border-code` | Code block outline.                 |

### Color Intent

- **Dark mode** evokes a warm terminal: neutral gray surfaces (`#333`
  family) with *palegoldenrod* links, *lightskyblue* h1, *plum* inline
  code, *pink* emphasis. Hue variety is intentional — each semantic role
  gets a distinct color so readers can scan structure.
- **Light mode** keeps the same semantic contrasts but shifts to muted
  parchment grays (`#f5f5f5` family) with desaturated gold, navy blue, and
  mulberry. It should feel like a printed reference page.

When adding colors, preserve this variety — do not collapse everything to
a single accent hue.

## Layout Tokens

Defined once in `:root` and inherited by light mode:

| Token                | Default  | Role                                    |
| -------------------- | -------- | --------------------------------------- |
| `--layout-max-width` | `1280px` | Outer container cap.                    |
| `--content-width`    | `960px`  | Prose column cap.                       |
| `--sidebar-width`    | `280px`  | Fixed sidebar width.                    |
| `--header-height`    | `48px`   | Fixed top header.                       |
| `--line-height`      | `1.6`    | Body text line height.                  |

Responsive breakpoints are 768px (sidebar becomes a drawer) and 480px
(heading sizes drop). Preserve both.

## Do / Don't

**Do**
- Add new semantic tokens when introducing a new role — never hard-code a
  color in a component rule.
- Keep `:root` and `[data-theme="light"]` in sync.
- Test changes in both dark and light mode and at mobile width.

**Don't**
- Don't introduce webfonts, icon fonts, or external CSS imports.
- Don't add gradients, drop-shadows beyond the existing soft image shadow,
  or animations beyond simple hover fades.
- **Don't edit code-highlight colors in CSS.** Syntax highlighting is
  handled by **syntect** via the `[highlight]` section of `config.toml`.
  Change `dark_theme` / `light_theme` there instead.
- Don't remove or rename existing tokens without updating every consumer
  in `main.css` and the templates in `bases/standard/templates/`.

## When Modifying This Theme

1. Read this file first.
2. Verify changes in both dark and light modes.
3. If you add a new token or rule, document it here.
