# Monotone Theme — Design Specification

This document describes the design intent of the `monotone` theme for
docs-gen. AI coding agents should read this file before modifying
`static/css/main.css` or `config.toml`, so that changes stay consistent
with the theme's character.

## Design Philosophy

The `monotone` theme is aimed at **serious, long-form technical writing**
— API references, research notes, engineering documentation — where the
design should disappear behind the prose. It is **strictly grayscale**: no
hue ever enters the palette. The visual tone is closer to a typeset book or
a high-end editorial site than to a developer portal.

- **Grayscale, always.** The only colors used are neutral grays. If a role
  seems to need a hue, reconsider the role.
- **Structure through typography, not color.** Hierarchy is communicated
  via weight, letter-spacing, and borders — not color accents.
- **Generous whitespace.** Line height is 1.7 (vs 1.6 in `default`).
  Content padding is larger. H2 has more top margin.
- **Subtle transitions.** 0.15s color fades on links, buttons, and nav
  items — more polished than `default`, but never flashy.
- **Tokens over hard-coded values.** Any gray that might reasonably change
  is a CSS custom property on `:root`.

## Color Tokens

All colors are CSS custom properties. Dark mode values live in `:root`;
light mode overrides live in `[data-theme="light"]`. **Every token defined
in one must be defined in the other — and every value must be grayscale.**

### Surfaces
| Token            | Role                                              |
| ---------------- | ------------------------------------------------- |
| `--bg`           | Page background. Near-black in dark, bone in light. |
| `--bg-secondary` | Header, search modal, table header, hover fills. |
| `--bg-code`      | Inline and block code backgrounds.                |

### Text
| Token                | Role                                          |
| -------------------- | --------------------------------------------- |
| `--text`             | Body copy default. Medium gray.               |
| `--text-bright`      | Highest-contrast text (h3, active nav, input).|
| `--text-muted`       | Sidebar items, footer, captions, blockquote.  |
| `--text-code`        | Code block foreground.                        |
| `--text-inline-code` | Inline `` `code` `` foreground.               |

### Accents
| Token                  | Role                                         |
| ---------------------- | -------------------------------------------- |
| `--link`               | Inline link. Near `--text-bright`; distinction comes from underline, not hue. |
| `--heading`            | `h1` — near-white/near-black. Thin weight.   |
| `--heading-link`       | `h2` — slightly muted, underlined by border. |
| `--emphasis`           | `strong`, active nav. Max contrast, still gray. |
| `--header-nav-link`    | Top-navigation links.                        |
| `--nav-section`        | Sidebar section label (inactive).            |
| `--nav-section-active` | Sidebar section label (current page).        |

### Borders
| Token           | Role                                 |
| --------------- | ------------------------------------ |
| `--border`      | Default separator (header, h2, table, modal). Used more prominently than in `default`. |
| `--border-code` | Code block outline.                  |

### Color Intent

- **Dark mode** is a deep, warm charcoal (`#1c1c1c` / `#232323`) with
  bright near-white text at the top of the scale (`#e8e8e8`) and
  mid-grays for body (`#b8b8b8`). Links and headings sit near the top of
  the scale; the difference from body text is about **brightness**, not
  color.
- **Light mode** is a warm paper tone (`#f4f3f1` / `#eae8e5`) with
  near-black text (`#1a1a1a`) for headings and `#3a3a3a` for body. Think
  editorial print, not flat pure-white web UI.

## Layout Tokens

Defined once in `:root` and inherited by light mode:

| Token                | Default  | Role                                  |
| -------------------- | -------- | ------------------------------------- |
| `--layout-max-width` | `1280px` | Outer container cap.                  |
| `--content-width`    | `960px`  | Prose column cap.                     |
| `--sidebar-width`    | `280px`  | Fixed sidebar width.                  |
| `--header-height`    | `48px`   | Fixed top header.                     |
| `--line-height`      | `1.7`    | **Wider** than `default` on purpose.  |

Content padding is also deliberately larger than `default` (`40px 32px`
instead of `32px 24px`). Responsive breakpoints are 768px and 480px.

## Signature Elements

These are visual moves that define `monotone` — **don't remove them
without a very good reason**:

- **H2 bottom border** using `--border`.
- **Uppercase sidebar section titles** with letter-spacing.
- **Underlined article links** with `--text-muted` that darken on hover.
- **Gray draft banner (`#666`)** instead of red — staying true to the
  no-hue rule.
- **H1 thin weight (300)** and slight negative letter-spacing.
- **Helvetica Neue preference** via the font stack, with font smoothing
  enabled.

## Do / Don't

**Do**
- Add new semantic tokens when introducing a new role — never hard-code
  a color in a component rule.
- Keep `:root` and `[data-theme="light"]` in sync.
- Use font weight, letter-spacing, and borders — not color — to build
  hierarchy.
- Test changes in both dark and light mode and at mobile width.

**Don't**
- **Don't introduce any hue.** No blues, greens, reds, oranges, yellows.
  Grayscale only, including the draft banner and search highlight.
- Don't introduce webfonts, icon fonts, or external CSS imports.
- Don't use gradients or heavy drop-shadows.
- **Don't edit code-highlight colors in CSS.** Syntax highlighting is
  handled by **syntect** via the `[highlight]` section of `config.toml`.
  (Note: syntect themes *do* contain hues — this is the single permitted
  exception, scoped to code block interiors only.)
- Don't remove or rename existing tokens without updating every consumer
  in `main.css` and the templates in `bases/standard/templates/`.

## When Modifying This Theme

1. Read this file first.
2. Verify changes in both dark and light modes.
3. **Verify no hue has crept in.** A grayscale screenshot should be
   visually identical to the original.
4. If you add a new token or signature element, document it here.
