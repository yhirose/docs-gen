use crate::config::SiteConfig;
use crate::markdown::{Frontmatter, MarkdownRenderer};
use anyhow::{Context, Result};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ── Diagnostic types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub file: String,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tag = match self.severity {
            Severity::Warning => "[warn]",
            Severity::Error => "[error]",
        };
        write!(f, "{} {}: {}", tag, self.file, self.message)
    }
}

// ── Lightweight page info (no HTML rendering) ───────────────────────────────

#[derive(Debug)]
struct CheckPage {
    frontmatter: Frontmatter,
    body: String,
    /// Relative path from the pages/<lang>/ directory, e.g. "guide/01-getting-started.md"
    rel_path: String,
    /// Absolute path to the source .md file
    abs_path: PathBuf,
    /// Section name extracted from directory structure
    section: String,
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Run all checks on the given source directory.
/// Returns `Ok(true)` if errors were found, `Ok(false)` if clean.
pub fn run(src: &Path) -> Result<bool> {
    let config = SiteConfig::load(src, None)?;

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let default_lang = config.system.langs.first().map(|s| s.as_str()).unwrap_or("en");
    let default_pages_dir = src.join("pages").join(default_lang);

    for lang in &config.system.langs {
        let pages_dir = src.join("pages").join(lang);
        if !pages_dir.exists() {
            eprintln!("Warning: pages directory not found for lang '{}', skipping", lang);
            continue;
        }

        let pages = collect_check_pages(&pages_dir)?;

        // Order checks
        check_order_duplicates(&pages, lang, &mut diagnostics);
        check_order_unset(&pages, lang, &mut diagnostics);

        // Link checks — for non-default languages, fall back to default lang dir for images
        let fallback_dir = if lang != default_lang { Some(default_pages_dir.as_path()) } else { None };
        check_internal_links(&pages, &pages_dir, lang, fallback_dir, &mut diagnostics);

        // Unreferenced page check
        check_unreferenced_pages(&pages, &pages_dir, lang, &mut diagnostics);
    }

    // Report
    let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();

    for d in &diagnostics {
        eprintln!("{}", d);
    }

    if errors > 0 || warnings > 0 {
        eprintln!();
        eprintln!(
            "{} error(s), {} warning(s) found.",
            errors, warnings
        );
    } else {
        println!("All checks passed.");
    }

    Ok(errors > 0)
}

// ── Page collection ─────────────────────────────────────────────────────────

fn collect_check_pages(pages_dir: &Path) -> Result<Vec<CheckPage>> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(pages_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let path = entry.path();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let (frontmatter, body) = MarkdownRenderer::parse_frontmatter(&content)
            .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;

        let rel = path.strip_prefix(pages_dir)?;
        let rel_str = rel.to_string_lossy().to_string();

        // Section = first directory component, or empty for root files
        let section = rel
            .components()
            .next()
            .and_then(|c| {
                let s = c.as_os_str().to_string_lossy().to_string();
                if s.ends_with(".md") {
                    None // root-level file, no section
                } else {
                    Some(s)
                }
            })
            .unwrap_or_default();

        pages.push(CheckPage {
            frontmatter,
            body: body.to_string(),
            rel_path: rel_str,
            abs_path: path.to_path_buf(),
            section,
        });
    }

    Ok(pages)
}

// ── Order checks ────────────────────────────────────────────────────────────

fn check_order_duplicates(pages: &[CheckPage], lang: &str, diags: &mut Vec<Diagnostic>) {
    // Group non-index pages by section
    let mut section_orders: HashMap<&str, HashMap<i32, Vec<&str>>> = HashMap::new();

    for page in pages {
        if page.section.is_empty() {
            continue; // root-level pages (e.g. index.md)
        }
        if page.rel_path.ends_with("index.md") {
            continue; // section index pages don't participate in order checks
        }
        section_orders
            .entry(&page.section)
            .or_default()
            .entry(page.frontmatter.order)
            .or_default()
            .push(&page.rel_path);
    }

    for (section, orders) in &section_orders {
        for (order, files) in orders {
            if files.len() > 1 {
                let file_list = files.join(", ");
                diags.push(Diagnostic {
                    severity: Severity::Warning,
                    file: format!("[{}] {}/", lang, section),
                    message: format!(
                        "duplicate order {} in: {}",
                        order, file_list
                    ),
                });
            }
        }
    }
}

fn check_order_unset(pages: &[CheckPage], lang: &str, diags: &mut Vec<Diagnostic>) {
    for page in pages {
        if page.section.is_empty() {
            continue;
        }
        if page.rel_path.ends_with("index.md") {
            continue;
        }
        if page.frontmatter.order == 0 {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                file: format!("[{}] {}", lang, page.rel_path),
                message: "order is not set (defaults to 0)".to_string(),
            });
        }
    }
}

// ── Link checks ─────────────────────────────────────────────────────────────

fn check_internal_links(
    pages: &[CheckPage],
    pages_dir: &Path,
    lang: &str,
    fallback_pages_dir: Option<&Path>,
    diags: &mut Vec<Diagnostic>,
) {
    for page in pages {
        let links = extract_links(&page.body);

            // Non-index pages render as slug/index.html, so relative links in
            // the rendered site are resolved from a virtual subdirectory.
            // e.g. guide/01-intro.md → guide/01-intro/index.html
            //       link "../02-other/" resolves from guide/01-intro/ → guide/02-other/
            let page_dir = if page.rel_path.ends_with("index.md") {
                page.abs_path.parent().unwrap_or(pages_dir).to_path_buf()
            } else {
                let stem = page.abs_path.file_stem().unwrap_or_default();
                page.abs_path.parent().unwrap_or(pages_dir).join(stem)
            };

        for (dest, is_image) in links {
            // Skip external links
            if dest.starts_with("http://")
                || dest.starts_with("https://")
                || dest.starts_with("mailto:")
                || dest.starts_with("tel:")
                || dest.starts_with('#')
            {
                continue;
            }

            // Strip anchor fragment
            let dest_no_anchor = dest.split('#').next().unwrap_or(&dest);
            if dest_no_anchor.is_empty() {
                continue;
            }

            // Normalize the joined path (page_dir may be a virtual directory
            // that doesn't exist on disk, so we must resolve ".." ourselves).
            let resolved = normalize_path(&page_dir.join(dest_no_anchor));

            let exists = if dest_no_anchor.ends_with('/') {
                // Directory link → check for index.md first
                if resolved.join("index.md").exists() {
                    true
                } else {
                    // docs-gen renders foo.md → foo/index.html, so a link to
                    // "foo/" is valid if "foo.md" exists as a sibling.
                    let trimmed = dest_no_anchor.trim_end_matches('/');
                    let as_md = normalize_path(&page_dir.join(format!("{}.md", trimmed)));
                    as_md.exists()
                }
            } else if resolved.exists() {
                true
            } else {
                // Maybe it's a page reference without extension → try .md
                resolved.with_extension("md").exists()
            };

            if !exists {
                // For non-default languages, check if image exists in default lang dir
                let fallback_exists = if is_image {
                    if let Some(fb_dir) = fallback_pages_dir {
                        // Reconstruct the path relative to pages_dir, then check in fallback
                        if let Ok(rel) = resolved.strip_prefix(pages_dir) {
                            let fb_path = fb_dir.join(rel);
                            fb_path.exists()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !fallback_exists {
                    let severity = if is_image {
                        Severity::Warning
                    } else {
                        Severity::Error
                    };
                    let kind = if is_image { "image" } else { "link" };
                    diags.push(Diagnostic {
                        severity,
                        file: format!("[{}] {}", lang, page.rel_path),
                        message: format!("broken {} target: {}", kind, dest),
                    });
                }
            }
        }
    }
}

// ── Unreferenced page check ─────────────────────────────────────────────────

fn check_unreferenced_pages(
    pages: &[CheckPage],
    pages_dir: &Path,
    lang: &str,
    diags: &mut Vec<Diagnostic>,
) {
    // Collect all page rel_paths (as targets to match against)
    let all_pages: HashSet<String> = pages.iter().map(|p| p.rel_path.clone()).collect();

    // Collect all resolved link targets across all pages
    let mut referenced: HashSet<String> = HashSet::new();

    for page in pages {
        let links = extract_links(&page.body);

        let page_dir = if page.rel_path.ends_with("index.md") {
            page.abs_path.parent().unwrap_or(pages_dir).to_path_buf()
        } else {
            let stem = page.abs_path.file_stem().unwrap_or_default();
            page.abs_path.parent().unwrap_or(pages_dir).join(stem)
        };

        for (dest, _is_image) in links {
            if dest.starts_with("http://")
                || dest.starts_with("https://")
                || dest.starts_with("mailto:")
                || dest.starts_with("tel:")
                || dest.starts_with('#')
            {
                continue;
            }

            let dest_no_anchor = dest.split('#').next().unwrap_or(&dest);
            if dest_no_anchor.is_empty() {
                continue;
            }

            let resolved = normalize_path(&page_dir.join(dest_no_anchor));

            // Try to map resolved absolute path back to a rel_path
            if let Ok(rel) = resolved.strip_prefix(pages_dir) {
                if dest_no_anchor.ends_with('/') {
                    // Directory link → index.md or sibling .md
                    let index_path = rel.join("index.md").to_string_lossy().to_string();
                    if all_pages.contains(&index_path) {
                        referenced.insert(index_path);
                    } else {
                        let trimmed = dest_no_anchor.trim_end_matches('/');
                        let as_md = normalize_path(&page_dir.join(format!("{}.md", trimmed)));
                        if let Ok(md_rel) = as_md.strip_prefix(pages_dir) {
                            let md_rel_str = md_rel.to_string_lossy().to_string();
                            if all_pages.contains(&md_rel_str) {
                                referenced.insert(md_rel_str);
                            }
                        }
                    }
                } else {
                    let rel_str = rel.to_string_lossy().to_string();
                    if all_pages.contains(&rel_str) {
                        referenced.insert(rel_str);
                    } else {
                        // Try with .md extension
                        let with_md = format!("{}.md", rel_str);
                        if all_pages.contains(&with_md) {
                            referenced.insert(with_md);
                        }
                    }
                }
            }
        }
    }

    // Report pages not referenced by any link
    for page in pages {
        // Skip index.md files (root portal and section indices are
        // reachable via nav buttons and sidebar, not explicit links)
        if page.rel_path.ends_with("index.md") {
            continue;
        }

        if !referenced.contains(&page.rel_path) {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                file: format!("[{}] {}", lang, page.rel_path),
                message: "page is not referenced by any link".to_string(),
            });
        }
    }
}

/// Normalize a path by resolving `.` and `..` components without touching the
/// filesystem.  This is needed because the "virtual directory" for non-index
/// pages doesn't actually exist on disk, so `fs::canonicalize` would fail.
fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            _ => out.push(comp),
        }
    }
    out
}

/// Extract link and image destinations from Markdown body.
/// Returns a list of (destination, is_image).
fn extract_links(markdown: &str) -> Vec<(String, bool)> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(markdown, options);
    let mut links = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                links.push((dest_url.to_string(), false));
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                links.push((dest_url.to_string(), true));
            }
            _ => {}
        }
    }

    links
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page(rel_path: &str, section: &str, order: i32, body: &str) -> CheckPage {
        CheckPage {
            frontmatter: Frontmatter {
                title: "Test".to_string(),
                order,
                status: None,
            },
            body: body.to_string(),
            rel_path: rel_path.to_string(),
            abs_path: PathBuf::from(rel_path),
            section: section.to_string(),
        }
    }

    #[test]
    fn test_order_duplicate_detected() {
        let pages = vec![
            make_page("guide/01-first.md", "guide", 1, ""),
            make_page("guide/02-second.md", "guide", 1, ""),
            make_page("guide/03-third.md", "guide", 2, ""),
        ];
        let mut diags = Vec::new();
        check_order_duplicates(&pages, "en", &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert!(diags[0].message.contains("duplicate order 1"));
    }

    #[test]
    fn test_order_no_duplicate() {
        let pages = vec![
            make_page("guide/01-first.md", "guide", 1, ""),
            make_page("guide/02-second.md", "guide", 2, ""),
        ];
        let mut diags = Vec::new();
        check_order_duplicates(&pages, "en", &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_order_unset_detected() {
        let pages = vec![
            make_page("guide/page.md", "guide", 0, ""),
            make_page("guide/other.md", "guide", 1, ""),
        ];
        let mut diags = Vec::new();
        check_order_unset(&pages, "en", &mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("order is not set"));
    }

    #[test]
    fn test_order_index_md_excluded() {
        let pages = vec![
            make_page("guide/index.md", "guide", 0, ""),
        ];
        let mut diags = Vec::new();
        check_order_unset(&pages, "en", &mut diags);
        assert!(diags.is_empty());
        check_order_duplicates(&pages, "en", &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_extract_links_basic() {
        let md = "[click](../other/) and ![img](pic.png)";
        let links = extract_links(md);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], ("../other/".to_string(), false));
        assert_eq!(links[1], ("pic.png".to_string(), true));
    }

    #[test]
    fn test_extract_links_external_skipped_in_check() {
        let md = "[ext](https://example.com) [int](../page/)";
        let links = extract_links(md);
        // extract_links returns all links; filtering is done in check_internal_links
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_broken_link_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        fs::create_dir_all(&guide_dir).unwrap();
        let page_path = guide_dir.join("page.md");
        fs::write(&page_path, "").unwrap();

        let pages = vec![CheckPage {
            frontmatter: Frontmatter {
                title: "Test".to_string(),
                order: 1,
                status: None,
            },
            body: "[broken](../nonexistent/)".to_string(),
            rel_path: "guide/page.md".to_string(),
            abs_path: page_path,
            section: "guide".to_string(),
        }];

        let mut diags = Vec::new();
        check_internal_links(&pages, tmp.path(), "en", None, &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
        assert!(diags[0].message.contains("broken link"));
    }

    #[test]
    fn test_unreferenced_page_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        // Create two pages: index links to page1 but not page2
        let index_path = guide_dir.join("index.md");
        let page1_path = guide_dir.join("01-first.md");
        let page2_path = guide_dir.join("02-second.md");
        fs::write(&index_path, "").unwrap();
        fs::write(&page1_path, "").unwrap();
        fs::write(&page2_path, "").unwrap();

        let pages = vec![
            CheckPage {
                frontmatter: Frontmatter { title: "Guide".into(), order: 0, status: None },
                body: "[First](01-first/)".to_string(),
                rel_path: "guide/index.md".to_string(),
                abs_path: index_path,
                section: "guide".to_string(),
            },
            CheckPage {
                frontmatter: Frontmatter { title: "First".into(), order: 1, status: None },
                body: "".to_string(),
                rel_path: "guide/01-first.md".to_string(),
                abs_path: page1_path,
                section: "guide".to_string(),
            },
            CheckPage {
                frontmatter: Frontmatter { title: "Second".into(), order: 2, status: None },
                body: "".to_string(),
                rel_path: "guide/02-second.md".to_string(),
                abs_path: page2_path,
                section: "guide".to_string(),
            },
        ];

        let mut diags = Vec::new();
        check_unreferenced_pages(&pages, tmp.path(), "en", &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert!(diags[0].message.contains("not referenced"));
        assert!(diags[0].file.contains("02-second.md"));
    }

    #[test]
    fn test_unreferenced_page_index_excluded() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        let index_path = tmp.path().join("index.md");
        let guide_index_path = guide_dir.join("index.md");
        fs::write(&index_path, "").unwrap();
        fs::write(&guide_index_path, "").unwrap();

        // Neither index.md is linked, but both should be excluded
        let pages = vec![
            CheckPage {
                frontmatter: Frontmatter { title: "Home".into(), order: 0, status: None },
                body: "".to_string(),
                rel_path: "index.md".to_string(),
                abs_path: index_path,
                section: "".to_string(),
            },
            CheckPage {
                frontmatter: Frontmatter { title: "Guide".into(), order: 0, status: None },
                body: "".to_string(),
                rel_path: "guide/index.md".to_string(),
                abs_path: guide_index_path,
                section: "guide".to_string(),
            },
        ];

        let mut diags = Vec::new();
        check_unreferenced_pages(&pages, tmp.path(), "en", &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_all_pages_referenced_no_warning() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        let index_path = guide_dir.join("index.md");
        let page1_path = guide_dir.join("01-first.md");
        let page2_path = guide_dir.join("02-second.md");
        fs::write(&index_path, "").unwrap();
        fs::write(&page1_path, "").unwrap();
        fs::write(&page2_path, "").unwrap();

        let pages = vec![
            CheckPage {
                frontmatter: Frontmatter { title: "Guide".into(), order: 0, status: None },
                body: "[First](01-first/) [Second](02-second/)".to_string(),
                rel_path: "guide/index.md".to_string(),
                abs_path: index_path,
                section: "guide".to_string(),
            },
            CheckPage {
                frontmatter: Frontmatter { title: "First".into(), order: 1, status: None },
                body: "[Second](../02-second/)".to_string(),
                rel_path: "guide/01-first.md".to_string(),
                abs_path: page1_path,
                section: "guide".to_string(),
            },
            CheckPage {
                frontmatter: Frontmatter { title: "Second".into(), order: 2, status: None },
                body: "".to_string(),
                rel_path: "guide/02-second.md".to_string(),
                abs_path: page2_path,
                section: "guide".to_string(),
            },
        ];

        let mut diags = Vec::new();
        check_unreferenced_pages(&pages, tmp.path(), "en", &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_colocated_image_link_valid() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        let page_path = guide_dir.join("page.md");
        fs::write(&page_path, "").unwrap();
        fs::write(guide_dir.join("screenshot.png"), "fake-png").unwrap();

        // Non-index page renders as guide/page/index.html
        // Image reference ./screenshot.png resolves from guide/page/ → guide/page/screenshot.png
        // But the actual file is at guide/screenshot.png
        // So the correct reference from a non-index page is ../screenshot.png
        let pages = vec![CheckPage {
            frontmatter: Frontmatter { title: "Test".into(), order: 1, status: None },
            body: "![img](../screenshot.png)".to_string(),
            rel_path: "guide/page.md".to_string(),
            abs_path: page_path,
            section: "guide".to_string(),
        }];

        let mut diags = Vec::new();
        check_internal_links(&pages, tmp.path(), "en", None, &mut diags);
        assert!(diags.is_empty(), "Valid colocated image should not produce diagnostics, got: {:?}", diags);
    }

    #[test]
    fn test_colocated_image_link_broken() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        let page_path = guide_dir.join("page.md");
        fs::write(&page_path, "").unwrap();
        // No screenshot.png file exists

        let pages = vec![CheckPage {
            frontmatter: Frontmatter { title: "Test".into(), order: 1, status: None },
            body: "![img](../nonexistent.png)".to_string(),
            rel_path: "guide/page.md".to_string(),
            abs_path: page_path,
            section: "guide".to_string(),
        }];

        let mut diags = Vec::new();
        check_internal_links(&pages, tmp.path(), "en", None, &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert!(diags[0].message.contains("broken image"));
    }

    #[test]
    fn test_valid_link_no_diagnostic() {
        let tmp = tempfile::tempdir().unwrap();
        let guide_dir = tmp.path().join("guide");
        let other_dir = tmp.path().join("other");
        fs::create_dir_all(&guide_dir).unwrap();
        fs::create_dir_all(&other_dir).unwrap();

        let page_path = guide_dir.join("page.md");
        fs::write(&page_path, "").unwrap();
        fs::write(other_dir.join("index.md"), "---\ntitle: Other\n---\n").unwrap();

        // Non-index page "guide/page.md" renders as "guide/page/index.html",
        // so a link to the "other" section needs ../../other/ (up from page/ then guide/).
        let pages = vec![CheckPage {
            frontmatter: Frontmatter {
                title: "Test".to_string(),
                order: 1,
                status: None,
            },
            body: "[valid](../../other/)".to_string(),
            rel_path: "guide/page.md".to_string(),
            abs_path: page_path,
            section: "guide".to_string(),
        }];

        let mut diags = Vec::new();
        check_internal_links(&pages, tmp.path(), "en", None, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_image_fallback_to_default_lang() {
        let tmp = tempfile::tempdir().unwrap();

        // Set up default lang (en) with an image
        let en_guide = tmp.path().join("en").join("guide");
        fs::create_dir_all(&en_guide).unwrap();
        fs::write(en_guide.join("screenshot.png"), "fake-png").unwrap();

        // Set up non-default lang (ja) without the image
        let ja_guide = tmp.path().join("ja").join("guide");
        fs::create_dir_all(&ja_guide).unwrap();
        let page_path = ja_guide.join("page.md");
        fs::write(&page_path, "").unwrap();

        let ja_dir = tmp.path().join("ja");
        let en_dir = tmp.path().join("en");

        let pages = vec![CheckPage {
            frontmatter: Frontmatter { title: "Test".into(), order: 1, status: None },
            body: "![img](../screenshot.png)".to_string(),
            rel_path: "guide/page.md".to_string(),
            abs_path: page_path,
            section: "guide".to_string(),
        }];

        // Without fallback → warning
        let mut diags = Vec::new();
        check_internal_links(&pages, &ja_dir, "ja", None, &mut diags);
        assert_eq!(diags.len(), 1, "Should warn without fallback");

        // With fallback to en → no warning
        let mut diags = Vec::new();
        check_internal_links(&pages, &ja_dir, "ja", Some(&en_dir), &mut diags);
        assert!(diags.is_empty(), "Should not warn when image exists in default lang, got: {:?}", diags);
    }
}
