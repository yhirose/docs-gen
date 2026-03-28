use crate::config::{NavLink, SiteConfig};
use crate::defaults;
use crate::markdown::{Frontmatter, MarkdownRenderer};
use crate::utils::copy_dir_recursive;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct PageContext {
    title: String,
    url: String,
    status: Option<String>,
    canonical_url: Option<String>,
    alternate_langs: Vec<AlternateLang>,
}

#[derive(Debug, Serialize)]
struct AlternateLang {
    lang: String,
    url: String,
}

/// Entry for pages-data.json used by client-side search.
#[derive(Debug, Serialize)]
struct PageDataEntry {
    title: String,
    url: String,
    lang: String,
    section: String,
    /// Plain-text body with HTML tags stripped, optionally truncated to a configured limit.
    body: String,
}

#[derive(Debug, Serialize, Clone)]
struct NavItem {
    title: String,
    url: String,
    children: Vec<NavItem>,
    active: bool,
}

#[derive(Debug, Serialize)]
struct SiteContext {
    title: String,
    version: Option<String>,
    base_url: String,
    base_path: String,
    langs: Vec<String>,
    nav: Vec<NavLink>,
    footer_message: Option<String>,
    single_lang: bool,
}

struct Page {
    frontmatter: Frontmatter,
    html_content: String,
    url: String,
    out_path: PathBuf,
    rel_path: String,
    section: String,
}

struct ColocatedFile {
    src_path: PathBuf,
    out_path: PathBuf,
}

pub fn build(src: &Path, out: &Path, theme_override: Option<&str>) -> Result<()> {
    let config = SiteConfig::load(src, theme_override)?;
    let renderer = MarkdownRenderer::new(
        config.highlight_dark_theme(),
        config.highlight_light_theme(),
    );
    let single_lang = config.system.is_single_lang();

    // Build Tera: start with embedded defaults, then override with user templates
    let tera = build_tera(src, &config.system.theme, &config.base)?;

    // Clean output directory
    if out.exists() {
        fs::remove_dir_all(out).context("Failed to clean output directory")?;
    }
    fs::create_dir_all(out)?;

    // Copy static files in cascade order: builtin → user base → user style → site
    copy_default_static(out, &config.system.theme)?;
    // User-defined base static (if any)
    let user_base_static = src.join("bases").join(&config.base).join("static");
    if user_base_static.exists() {
        copy_dir_recursive(&user_base_static, out)?;
    }
    // User style static (if any)
    let style_static_dir = src.join("styles").join(&config.system.theme).join("static");
    if style_static_dir.exists() {
        copy_dir_recursive(&style_static_dir, out)?;
    }
    // Site-level static (if any)
    let static_dir = src.join("static");
    if static_dir.exists() {
        copy_dir_recursive(&static_dir, out)?;
    }

    // Collect page data entries for pages-data.json across all languages
    let mut page_data_entries: Vec<PageDataEntry> = Vec::new();

    // Colocated files from the primary (default) language, keyed by relative source path.
    // Maps to (source_path, output_suffix) for fallback copying to other languages.
    let default_lang = config.system.default_lang();
    let multi_lang = !single_lang;
    // page.url already includes base_path, so prepend only the hostname for absolute URLs.
    let hostname = config.site.hostname.as_ref().map(|h| h.trim_end_matches('/'));
    let mut primary_colocated: HashMap<PathBuf, (PathBuf, PathBuf)> = HashMap::new();

    // Build each language (default_lang is always first in config.system.langs)
    for lang in &config.system.langs {
        let is_primary = lang.as_str() == default_lang;
        let pages_dir = src.join("pages").join(lang);
        if !pages_dir.exists() {
            eprintln!("Warning: pages directory not found for lang '{}', skipping", lang);
            continue;
        }

        let pages = collect_pages(&pages_dir, lang, out, &renderer, &config.site.base_path, single_lang)?;

        // Copy colocated files (images, etc.) next to their pages
        let colocated = collect_colocated_files(&pages_dir, &pages)?;

        // Track which relative paths this language has (only needed for non-primary)
        let mut lang_rel_paths: HashSet<PathBuf> = HashSet::new();

        let lang_out_prefix = out.join(lang.as_str());

        for cf in &colocated {
            if let Some(parent) = cf.out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&cf.src_path, &cf.out_path)
                .with_context(|| format!("Failed to copy colocated file {}", cf.src_path.display()))?;

            if multi_lang {
                let rel = cf.src_path.strip_prefix(&pages_dir)
                    .unwrap_or(&cf.src_path)
                    .to_path_buf();

                if is_primary {
                    let out_suffix = cf.out_path.strip_prefix(&lang_out_prefix)
                        .unwrap_or(&cf.out_path)
                        .to_path_buf();
                    primary_colocated.insert(rel, (cf.src_path.clone(), out_suffix));
                } else {
                    lang_rel_paths.insert(rel);
                }
            }
        }

        // For non-primary languages, copy missing colocated files from primary language
        if multi_lang && !is_primary && !primary_colocated.is_empty() {
            for (rel, (primary_src, out_suffix)) in &primary_colocated {
                if !lang_rel_paths.contains(rel) {
                    let dest = out.join(lang.as_str()).join(out_suffix);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(primary_src, &dest)
                        .with_context(|| format!("Failed to copy fallback colocated file {}", primary_src.display()))?;
                }
            }
        }

        let nav = build_nav(&pages);

        for page in &pages {
            // Collect search data for pages-data.json
            let plain_body = strip_html_tags(&remove_light_theme_blocks(&page.html_content));
            let truncated_body = truncate_for_search(plain_body, config.system.search_max_chars);
            page_data_entries.push(PageDataEntry {
                title: page.frontmatter.title.clone(),
                url: page.url.clone(),
                lang: lang.clone(),
                section: page.section.clone(),
                body: truncated_body,
            });

            let template_name = if page.section.is_empty() {
                "portal.html"
            } else {
                "page.html"
            };

            // Filter nav to only the current section
            let section_nav: Vec<&NavItem> = nav
                .iter()
                .filter(|item| {
                    let item_section = extract_section(&item.url, &config.site.base_path, single_lang);
                    item_section == page.section
                })
                .collect();

            // lang_prefix: empty for single-language, "/en" etc. for multi-language
            let lang_prefix = if single_lang {
                String::new()
            } else {
                format!("/{}", lang)
            };

            let canonical_url = hostname.map(|h| format!("{}{}", h, page.url));

            let alternate_langs = match (hostname, multi_lang) {
                (Some(h), true) => build_alternate_langs(h, &page.url, lang, &config.system.langs, default_lang),
                _ => Vec::new(),
            };

            let mut ctx = tera::Context::new();
            ctx.insert("page", &PageContext {
                title: page.frontmatter.title.clone(),
                url: page.url.clone(),
                status: page.frontmatter.status.clone(),
                canonical_url,
                alternate_langs,
            });
            ctx.insert("content", &page.html_content);
            ctx.insert("lang", lang);
            ctx.insert("lang_prefix", &lang_prefix);
            ctx.insert("site", &SiteContext {
                title: config.site.title.clone(),
                version: config.site.version.clone(),
                base_url: config.site.base_url(),
                base_path: config.site.base_path.clone(),
                langs: config.system.langs.clone(),
                nav: config.nav.clone(),
                footer_message: config.site.footer_message.clone(),
                single_lang,
            });

            // Set active state and pass nav
            let mut nav_with_active: Vec<NavItem> = section_nav
                .into_iter()
                .cloned()
                .map(|mut item| {
                    set_active(&mut item, &page.url);
                    item
                })
                .collect();

            // If we're on a section index page, expand its children
            if let Some(item) = nav_with_active.first_mut() {
                if item.url == page.url {
                    item.active = true;
                }
            }

            ctx.insert("nav", &nav_with_active);

            let html = tera
                .render(template_name, &ctx)
                .with_context(|| format!("Failed to render template for {}", page.url))?;

            if let Some(parent) = page.out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&page.out_path, html)?;
        }
    }

    // Generate pages-data.json for client-side search
    let pages_json = serde_json::to_string(&page_data_entries)
        .context("Failed to serialize pages-data.json")?;
    fs::write(out.join("pages-data.json"), pages_json)?;

    // Generate root redirect
    generate_root_redirect(out, &config)?;

    // Generate sitemap.xml (only when hostname is configured)
    generate_sitemap(out, &config, &page_data_entries)?;

    println!(
        "Site generated: {} languages, output at {}",
        config.system.langs.len(),
        out.display()
    );

    Ok(())
}

fn collect_pages(
    pages_dir: &Path,
    lang: &str,
    out: &Path,
    renderer: &MarkdownRenderer,
    base_path: &str,
    single_lang: bool,
) -> Result<Vec<Page>> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(pages_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "md")
        })
    {
        let path = entry.path();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let (frontmatter, body) = MarkdownRenderer::parse_frontmatter(&content)
            .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;

        let html_content = renderer.render(body);

        let rel = path.strip_prefix(pages_dir)?;
        let rel_str = rel.to_string_lossy().to_string();

        // Compute URL and output path
        // Single-language: omit language directory (e.g. /guide/)
        // Multi-language: include language directory (e.g. /en/guide/)
        let (url, out_path) = if rel.file_name().map_or(false, |f| f == "index.md") {
            let parent = rel.parent().unwrap_or(Path::new(""));
            if parent.as_os_str().is_empty() {
                // Root index.md
                if single_lang {
                    (
                        format!("{}/", base_path),
                        out.join("index.html"),
                    )
                } else {
                    (
                        format!("{}/{}/", base_path, lang),
                        out.join(lang).join("index.html"),
                    )
                }
            } else {
                if single_lang {
                    (
                        format!("{}/{}/", base_path, parent.display()),
                        out.join(parent).join("index.html"),
                    )
                } else {
                    (
                        format!("{}/{}/{}/", base_path, lang, parent.display()),
                        out.join(lang).join(parent).join("index.html"),
                    )
                }
            }
        } else {
            let stem = rel.with_extension("");
            if single_lang {
                (
                    format!("{}/{}/", base_path, stem.display()),
                    out.join(&stem).join("index.html"),
                )
            } else {
                (
                    format!("{}/{}/{}/", base_path, lang, stem.display()),
                    out.join(lang).join(&stem).join("index.html"),
                )
            }
        };

        let section = extract_section(&url, base_path, single_lang);

        pages.push(Page {
            frontmatter,
            html_content,
            url,
            out_path,
            rel_path: rel_str,
            section,
        });
    }

    Ok(pages)
}

/// Collect non-Markdown files from the pages directory for colocation.
/// These files (images, etc.) are copied alongside their page's HTML output.
fn collect_colocated_files(
    pages_dir: &Path,
    pages: &[Page],
) -> Result<Vec<ColocatedFile>> {
    struct DirInfo {
        md_count: usize,
        has_index: bool,
        index_out_dir: Option<PathBuf>,
        page_out_dir: Option<PathBuf>,
    }

    let mut result = Vec::new();

    // Group pages by their source directory to determine output mapping
    let mut dirs: HashMap<PathBuf, DirInfo> = HashMap::new();

    for page in pages {
        let src_path = pages_dir.join(&page.rel_path);
        let dir = src_path.parent().unwrap_or(pages_dir).to_path_buf();

        let info = dirs.entry(dir).or_insert(DirInfo {
            md_count: 0,
            has_index: false,
            index_out_dir: None,
            page_out_dir: None,
        });
        info.md_count += 1;
        if page.rel_path.ends_with("index.md") {
            info.has_index = true;
            if let Some(out_dir) = page.out_path.parent() {
                info.index_out_dir = Some(out_dir.to_path_buf());
            }
        }
        // out_path is e.g. out/guide/01-intro/index.html — parent gives the page's output dir
        if let Some(page_out_dir) = page.out_path.parent() {
            info.page_out_dir = Some(page_out_dir.to_path_buf());
        }
    }

    // Walk the pages directory for non-.md files
    for entry in WalkDir::new(pages_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip .md files (they are pages)
        if path.extension().map_or(false, |ext| ext == "md") {
            continue;
        }

        // Skip hidden files
        if path
            .file_name()
            .map_or(false, |f| f.to_string_lossy().starts_with('.'))
        {
            continue;
        }

        let file_dir = path.parent().unwrap_or(pages_dir).to_path_buf();
        let file_name = path.file_name().unwrap();

        // Determine output directory for this colocated file
        let out_dir = if let Some(info) = dirs.get(&file_dir) {
            if info.has_index {
                // Directory has index.md → copy to the index page's output directory
                info.index_out_dir.clone()
            } else if info.md_count == 1 {
                // Single .md file → copy into that page's output directory
                info.page_out_dir.clone()
            } else {
                // Multiple .md files → copy to the parent (section) directory
                info.page_out_dir
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
            }
        } else {
            // No .md files in this directory (orphan file in a subdirectory) — skip
            None
        };

        if let Some(out_dir) = out_dir {
            result.push(ColocatedFile {
                src_path: path.to_path_buf(),
                out_path: out_dir.join(file_name),
            });
        }
    }

    Ok(result)
}

fn extract_section(url: &str, base_path: &str, single_lang: bool) -> String {
    // Strip base_path prefix before parsing
    let stripped = url.strip_prefix(base_path).unwrap_or(url);
    let parts: Vec<&str> = stripped.trim_matches('/').split('/').collect();
    if single_lang {
        // URL format: /section/... (no lang prefix)
        if !parts.is_empty() && !parts[0].is_empty() {
            parts[0].to_string()
        } else {
            String::new()
        }
    } else {
        // URL format: /<lang>/section/...
        if parts.len() >= 2 {
            parts[1].to_string()
        } else {
            String::new()
        }
    }
}

fn build_nav(pages: &[Page]) -> Vec<NavItem> {
    // Group pages by section (top-level directory)
    let mut sections: std::collections::BTreeMap<String, Vec<&Page>> =
        std::collections::BTreeMap::new();

    for page in pages {
        if page.section.is_empty() {
            continue; // Skip root index (portal)
        }
        sections
            .entry(page.section.clone())
            .or_default()
            .push(page);
    }

    let mut nav = Vec::new();

    for (section, mut section_pages) in sections {
        // Sort by order, then by filename
        section_pages.sort_by(|a, b| {
            a.frontmatter
                .order
                .cmp(&b.frontmatter.order)
                .then_with(|| a.rel_path.cmp(&b.rel_path))
        });

        // Find the section index page
        let index_page = section_pages
            .iter()
            .find(|p| p.rel_path.ends_with("index.md") && p.section == section);

        let section_title = index_page
            .map(|p| p.frontmatter.title.clone())
            .unwrap_or_else(|| section.clone());
        let section_url = index_page
            .map(|p| p.url.clone())
            .unwrap_or_default();

        let children: Vec<NavItem> = section_pages
            .iter()
            .filter(|p| !p.rel_path.ends_with("index.md") || p.section != section)
            .map(|p| NavItem {
                title: p.frontmatter.title.clone(),
                url: p.url.clone(),
                children: Vec::new(),
                active: false,
            })
            .collect();

        nav.push(NavItem {
            title: section_title,
            url: section_url,
            children,
            active: false,
        });
    }

    // Sort nav sections by order of their index pages
    nav
}

fn set_active(item: &mut NavItem, current_url: &str) {
    if item.url == current_url {
        item.active = true;
    }
    for child in &mut item.children {
        set_active(child, current_url);
        if child.active {
            item.active = true;
        }
    }
}

fn generate_root_redirect(out: &Path, config: &SiteConfig) -> Result<()> {
    // Single-language: root index.html is the actual page content, no redirect needed
    if config.system.is_single_lang() {
        return Ok(());
    }

    let base_path = &config.site.base_path;
    let lang_links: String = config
        .system
        .langs
        .iter()
        .map(|l| format!(r#"<li><a href="{base_path}/{lang}/">{lang}</a></li>"#, base_path = base_path, lang = l))
        .collect::<Vec<_>>()
        .join("\n");
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<script>
(function() {{
  var lang = localStorage.getItem('preferred-lang') || '{default_lang}';
  window.location.replace('{base_path}/' + lang + '/');
}})();
</script>
<meta http-equiv="refresh" content="0;url={base_path}/{default_lang}/">
<title>Redirecting...</title>
</head>
<body>
<ul>
{lang_links}
</ul>
</body>
</html>"#,
        default_lang = config.system.default_lang(),
        base_path = base_path,
        lang_links = lang_links,
    );

    fs::write(out.join("index.html"), html)?;
    Ok(())
}

/// Generate sitemap.xml listing all pages with absolute URLs.
/// Only generated when `hostname` is configured (absolute URLs are required).
/// For multi-language sites, each URL includes `xhtml:link` alternates for hreflang.
fn generate_sitemap(out: &Path, config: &SiteConfig, pages: &[PageDataEntry]) -> Result<()> {
    if config.site.hostname.is_none() {
        return Ok(());
    }

    let hostname = config.site.hostname.as_ref().unwrap().trim_end_matches('/');
    let multi_lang = !config.system.is_single_lang();
    let default_lang = config.system.default_lang();

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"",
    );

    if multi_lang {
        xml.push_str("\n        xmlns:xhtml=\"http://www.w3.org/1999/xhtml\"");
    }

    xml.push_str(">\n");

    for page in pages {
        let loc = format!("{}{}", hostname, page.url);
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", escape_xml(&loc)));

        if multi_lang {
            for alt in build_alternate_langs(hostname, &page.url, &page.lang, &config.system.langs, default_lang) {
                xml.push_str(&format!(
                    "    <xhtml:link rel=\"alternate\" hreflang=\"{}\" href=\"{}\"/>\n",
                    alt.lang,
                    escape_xml(&alt.url)
                ));
            }
        }

        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");

    fs::write(out.join("sitemap.xml"), xml)?;
    Ok(())
}

/// Replace the language segment in a URL path (e.g. `/en/` → `/ja/`).
fn rewrite_lang_in_url(url: &str, from_lang: &str, to_lang: &str) -> String {
    url.replacen(&format!("/{}/", from_lang), &format!("/{}/", to_lang), 1)
}

/// Build alternate-language entries for hreflang, including `x-default`.
fn build_alternate_langs(hostname: &str, page_url: &str, current_lang: &str, langs: &[String], default_lang: &str) -> Vec<AlternateLang> {
    let mut alts: Vec<AlternateLang> = langs
        .iter()
        .map(|l| AlternateLang {
            lang: l.clone(),
            url: format!("{}{}", hostname, rewrite_lang_in_url(page_url, current_lang, l)),
        })
        .collect();
    alts.push(AlternateLang {
        lang: "x-default".to_string(),
        url: format!("{}{}", hostname, rewrite_lang_in_url(page_url, current_lang, default_lang)),
    });
    alts
}

/// Escape special XML characters in a string.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Register all `.html` templates from a directory into Tera (later registrations override earlier).
fn register_templates_from_dir(tera: &mut Tera, dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "html"))
    {
        let path = entry.path();
        let rel = path.strip_prefix(dir)?;
        let name = rel.to_string_lossy().replace('\\', "/");
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read template {}", path.display()))?;
        tera.add_raw_template(&name, &source)
            .with_context(|| format!("Failed to register template '{}'", name))?;
    }
    Ok(())
}

/// Build Tera with 4-level cascade: builtin (base+style) → user base → user style → site.
fn build_tera(src: &Path, theme_name: &str, base_name: &str) -> Result<Tera> {
    let mut tera = Tera::default();

    // 1. Builtin defaults (base + style already merged)
    for (name, source) in defaults::default_templates(theme_name) {
        tera.add_raw_template(name, source)
            .with_context(|| format!("Failed to add default template '{}'", name))?;
    }

    // 2. User-defined base templates (if any)
    register_templates_from_dir(&mut tera, &src.join("bases").join(base_name).join("templates"))?;

    // 3. User style templates (if any)
    register_templates_from_dir(&mut tera, &src.join("styles").join(theme_name).join("templates"))?;

    // 4. Site-level templates (if any)
    register_templates_from_dir(&mut tera, &src.join("templates"))?;

    Ok(tera)
}

/// Write embedded default static files (css/js) to the output directory.
fn copy_default_static(out: &Path, theme_name: &str) -> Result<()> {
    for (rel_path, content) in defaults::default_static_files(theme_name) {
        let target = out.join(rel_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        // Only write if not already present (user file takes precedence via
        // the subsequent copy_dir_recursive call, but write defaults first)
        fs::write(&target, content)?;
    }
    Ok(())
}

/// Strip HTML tags from a string and collapse whitespace into a single space,
/// producing a plain-text representation suitable for search indexing.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Insert a space to avoid words being glued across tags
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Collapse whitespace
    let collapsed: String = result.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
}

/// Remove `<div data-code-theme="light">...</div>` blocks so that
/// dual-theme code snippets are only indexed once.
fn remove_light_theme_blocks(html: &str) -> String {
    const MARKER: &str = "<div data-code-theme=\"light\"";
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(start) = remaining.find(MARKER) {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        let mut depth: usize = 0;
        let mut i = 0;
        while i < remaining.len() {
            if remaining[i..].starts_with("<div") {
                depth += 1;
                i += 4;
            } else if remaining[i..].starts_with("</div>") {
                depth -= 1;
                i += 6;
                if depth == 0 {
                    break;
                }
            } else {
                i += remaining[i..].chars().next().map_or(1, |c| c.len_utf8());
            }
        }
        remaining = &remaining[i..];
    }

    result.push_str(remaining);
    result
}

/// Truncate plain text for search indexing. If `max_chars` is 0, return the
/// full text; otherwise take at most `max_chars` characters.
fn truncate_for_search(mut text: String, max_chars: usize) -> String {
    // text.len() is byte length, which is always >= char count,
    // so this serves as a cheap early-out for short texts.
    if max_chars > 0 && text.len() > max_chars {
        if let Some((byte_idx, _)) = text.char_indices().nth(max_chars) {
            text.truncate(byte_idx);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags_basic() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
    }

    #[test]
    fn test_strip_html_tags_nested() {
        assert_eq!(strip_html_tags("<div><p>Hello <b>world</b></p></div>"), "Hello world");
    }

    #[test]
    fn test_strip_html_tags_collapse_whitespace() {
        let result = strip_html_tags("<p>Hello</p>  <p>World</p>");
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_strip_html_tags_empty() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_strip_html_tags_no_tags() {
        assert_eq!(strip_html_tags("plain text"), "plain text");
    }

    #[test]
    fn test_remove_light_theme_blocks() {
        let html = r#"<div data-code-theme="dark">dark code</div><div data-code-theme="light"><div>nested</div></div>after"#;
        let result = remove_light_theme_blocks(html);
        assert!(result.contains("dark code"));
        assert!(!result.contains("nested"));
        assert!(result.contains("after"));
    }

    #[test]
    fn test_remove_light_theme_blocks_no_light() {
        let html = "<div>no light theme here</div>";
        assert_eq!(remove_light_theme_blocks(html), html);
    }

    #[test]
    fn test_truncate_for_search_no_limit() {
        let text = "a".repeat(1000);
        let result = truncate_for_search(text.clone(), 0);
        assert_eq!(result.len(), 1000);
    }

    #[test]
    fn test_truncate_for_search_with_limit() {
        let text = "a".repeat(1000);
        let result = truncate_for_search(text, 100);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_truncate_for_search_shorter_than_limit() {
        let text = "hello".to_string();
        let result = truncate_for_search(text, 100);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_for_search_multibyte() {
        // 日本語: each char is multibyte but truncation counts chars, not bytes
        let text = "あいうえおかきくけこ".to_string(); // 10 chars
        let result = truncate_for_search(text, 5);
        assert_eq!(result, "あいうえお");
    }

    #[test]
    fn test_extract_section_single_lang() {
        assert_eq!(extract_section("/guide/start/", "", true), "guide");
        assert_eq!(extract_section("/", "", true), "");
    }

    #[test]
    fn test_extract_section_single_lang_with_base() {
        assert_eq!(extract_section("/docs/guide/start/", "/docs", true), "guide");
        assert_eq!(extract_section("/docs/", "/docs", true), "");
    }

    #[test]
    fn test_extract_section_multi_lang() {
        assert_eq!(extract_section("/en/guide/start/", "", false), "guide");
        assert_eq!(extract_section("/en/", "", false), "");
    }

    #[test]
    fn test_extract_section_multi_lang_with_base() {
        assert_eq!(extract_section("/docs/en/guide/", "/docs", false), "guide");
    }

    // ── Colocation tests ─────────────────────────────────────────

    #[test]
    fn test_colocated_single_md_copies_to_page_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let pages_dir = tmp.path().join("pages");
        let guide_dir = pages_dir.join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        // Single .md file + an image
        fs::write(guide_dir.join("01-intro.md"), "---\ntitle: Intro\n---\n").unwrap();
        fs::write(guide_dir.join("screenshot.png"), "fake-png").unwrap();

        let out_dir = tmp.path().join("out");
        let pages = vec![Page {
            frontmatter: Frontmatter { title: "Intro".into(), order: 1, status: None },
            html_content: String::new(),
            url: "/guide/01-intro/".into(),
            out_path: out_dir.join("guide").join("01-intro").join("index.html"),
            rel_path: "guide/01-intro.md".into(),
            section: "guide".into(),
        }];

        let colocated = collect_colocated_files(&pages_dir, &pages).unwrap();
        assert_eq!(colocated.len(), 1);
        assert_eq!(
            colocated[0].out_path,
            out_dir.join("guide").join("01-intro").join("screenshot.png")
        );
    }

    #[test]
    fn test_colocated_index_md_copies_to_section_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let pages_dir = tmp.path().join("pages");
        let guide_dir = pages_dir.join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        fs::write(guide_dir.join("index.md"), "---\ntitle: Guide\n---\n").unwrap();
        fs::write(guide_dir.join("diagram.png"), "fake-png").unwrap();

        let out_dir = tmp.path().join("out");
        let pages = vec![Page {
            frontmatter: Frontmatter { title: "Guide".into(), order: 0, status: None },
            html_content: String::new(),
            url: "/guide/".into(),
            out_path: out_dir.join("guide").join("index.html"),
            rel_path: "guide/index.md".into(),
            section: "guide".into(),
        }];

        let colocated = collect_colocated_files(&pages_dir, &pages).unwrap();
        assert_eq!(colocated.len(), 1);
        assert_eq!(
            colocated[0].out_path,
            out_dir.join("guide").join("diagram.png")
        );
    }

    #[test]
    fn test_colocated_multiple_md_copies_to_parent_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let pages_dir = tmp.path().join("pages");
        let guide_dir = pages_dir.join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        fs::write(guide_dir.join("01-intro.md"), "---\ntitle: Intro\n---\n").unwrap();
        fs::write(guide_dir.join("02-setup.md"), "---\ntitle: Setup\n---\n").unwrap();
        fs::write(guide_dir.join("screenshot.png"), "fake-png").unwrap();

        let out_dir = tmp.path().join("out");
        let pages = vec![
            Page {
                frontmatter: Frontmatter { title: "Intro".into(), order: 1, status: None },
                html_content: String::new(),
                url: "/guide/01-intro/".into(),
                out_path: out_dir.join("guide").join("01-intro").join("index.html"),
                rel_path: "guide/01-intro.md".into(),
                section: "guide".into(),
            },
            Page {
                frontmatter: Frontmatter { title: "Setup".into(), order: 2, status: None },
                html_content: String::new(),
                url: "/guide/02-setup/".into(),
                out_path: out_dir.join("guide").join("02-setup").join("index.html"),
                rel_path: "guide/02-setup.md".into(),
                section: "guide".into(),
            },
        ];

        let colocated = collect_colocated_files(&pages_dir, &pages).unwrap();
        assert_eq!(colocated.len(), 1);
        // Should be in guide/ (parent), not guide/01-intro/ or guide/02-setup/
        assert_eq!(
            colocated[0].out_path,
            out_dir.join("guide").join("screenshot.png")
        );
    }

    #[test]
    fn test_colocated_md_files_excluded() {
        let tmp = tempfile::tempdir().unwrap();
        let pages_dir = tmp.path().join("pages");
        let guide_dir = pages_dir.join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        fs::write(guide_dir.join("01-intro.md"), "---\ntitle: Intro\n---\n").unwrap();
        // No non-md files

        let out_dir = tmp.path().join("out");
        let pages = vec![Page {
            frontmatter: Frontmatter { title: "Intro".into(), order: 1, status: None },
            html_content: String::new(),
            url: "/guide/01-intro/".into(),
            out_path: out_dir.join("guide").join("01-intro").join("index.html"),
            rel_path: "guide/01-intro.md".into(),
            section: "guide".into(),
        }];

        let colocated = collect_colocated_files(&pages_dir, &pages).unwrap();
        assert!(colocated.is_empty());
    }

    #[test]
    fn test_colocated_hidden_files_excluded() {
        let tmp = tempfile::tempdir().unwrap();
        let pages_dir = tmp.path().join("pages");
        let guide_dir = pages_dir.join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        fs::write(guide_dir.join("01-intro.md"), "---\ntitle: Intro\n---\n").unwrap();
        fs::write(guide_dir.join(".hidden"), "secret").unwrap();
        fs::write(guide_dir.join("visible.png"), "fake-png").unwrap();

        let out_dir = tmp.path().join("out");
        let pages = vec![Page {
            frontmatter: Frontmatter { title: "Intro".into(), order: 1, status: None },
            html_content: String::new(),
            url: "/guide/01-intro/".into(),
            out_path: out_dir.join("guide").join("01-intro").join("index.html"),
            rel_path: "guide/01-intro.md".into(),
            section: "guide".into(),
        }];

        let colocated = collect_colocated_files(&pages_dir, &pages).unwrap();
        assert_eq!(colocated.len(), 1);
        assert!(colocated[0].src_path.to_string_lossy().contains("visible.png"));
    }

    #[test]
    fn test_colocated_multi_lang_output_path() {
        let tmp = tempfile::tempdir().unwrap();
        let pages_dir = tmp.path().join("pages");
        let guide_dir = pages_dir.join("guide");
        fs::create_dir_all(&guide_dir).unwrap();

        fs::write(guide_dir.join("01-intro.md"), "---\ntitle: Intro\n---\n").unwrap();
        fs::write(guide_dir.join("screenshot.png"), "fake-png").unwrap();

        // Multi-lang: output path includes lang directory
        let out_dir = tmp.path().join("out");
        let pages = vec![Page {
            frontmatter: Frontmatter { title: "Intro".into(), order: 1, status: None },
            html_content: String::new(),
            url: "/en/guide/01-intro/".into(),
            out_path: out_dir.join("en").join("guide").join("01-intro").join("index.html"),
            rel_path: "guide/01-intro.md".into(),
            section: "guide".into(),
        }];

        let colocated = collect_colocated_files(&pages_dir, &pages).unwrap();
        assert_eq!(colocated.len(), 1);
        assert_eq!(
            colocated[0].out_path,
            out_dir.join("en").join("guide").join("01-intro").join("screenshot.png")
        );
    }

    // ── SEO tests ────────────────────────────────────────────────

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("https://example.com/"), "https://example.com/");
        assert_eq!(escape_xml("<a&b>"), "&lt;a&amp;b&gt;");
        assert_eq!(escape_xml("\"x'y\""), "&quot;x&apos;y&quot;");
    }

    fn make_test_config(hostname: Option<&str>, langs: Vec<&str>, base_path: &str) -> SiteConfig {
        SiteConfig {
            system: crate::config::System {
                theme: "default".into(),
                langs: langs.into_iter().map(String::from).collect(),
                search_max_chars: 0,
            },
            site: crate::config::Site {
                title: "Test".into(),
                version: None,
                hostname: hostname.map(String::from),
                base_path: base_path.into(),
                footer_message: None,
            },
            highlight: None,
            base: crate::defaults::DEFAULT_BASE.into(),
            nav: vec![],
        }
    }

    #[test]
    fn test_generate_sitemap_skipped_without_hostname() {
        let tmp = tempfile::tempdir().unwrap();
        let config = make_test_config(None, vec!["en"], "");
        let pages = vec![PageDataEntry {
            title: "Home".into(), url: "/".into(), lang: "en".into(),
            section: String::new(), body: String::new(),
        }];
        generate_sitemap(tmp.path(), &config, &pages).unwrap();
        assert!(!tmp.path().join("sitemap.xml").exists());
    }

    #[test]
    fn test_generate_sitemap_multi_lang_with_hreflang() {
        let tmp = tempfile::tempdir().unwrap();
        let config = make_test_config(Some("https://example.com"), vec!["en", "ja"], "/docs");
        let pages = vec![
            PageDataEntry {
                title: "Home".into(), url: "/docs/en/".into(), lang: "en".into(),
                section: String::new(), body: String::new(),
            },
            PageDataEntry {
                title: "ホーム".into(), url: "/docs/ja/".into(), lang: "ja".into(),
                section: String::new(), body: String::new(),
            },
        ];
        generate_sitemap(tmp.path(), &config, &pages).unwrap();
        let content = fs::read_to_string(tmp.path().join("sitemap.xml")).unwrap();
        assert!(content.contains("<loc>https://example.com/docs/en/</loc>"));
        assert!(content.contains("<loc>https://example.com/docs/ja/</loc>"));
        assert!(content.contains("hreflang=\"en\""));
        assert!(content.contains("hreflang=\"ja\""));
        assert!(content.contains("hreflang=\"x-default\" href=\"https://example.com/docs/en/\""));
    }
}
