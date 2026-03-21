// Default embedded base, style, and scaffold files.
//
// Bases provide shared layout foundations (templates, JS, favicon).
// Styles reference a base and add style-specific files (config, CSS).
//
// Both `defaults/bases/` and `defaults/styles/` are embedded at compile time
// via `include_dir!`. Adding a new base or style only requires placing files
// under the appropriate directory — no manual constants needed.

use crate::config::StyleSystemConfig;
use include_dir::{include_dir, Dir};
use serde::Deserialize;

/// All built-in bases, embedded at compile time.
static BASES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/defaults/bases");

/// All built-in styles, embedded at compile time.
static STYLES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/defaults/styles");

/// All scaffold files (config.toml, sample pages), embedded at compile time.
static DEFAULTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/defaults");

/// Default base name used when a style's config.toml omits the `base` field.
pub const DEFAULT_BASE: &str = "standard";

/// Minimal struct used only to extract the `base` field from a style's config.toml.
#[derive(Deserialize)]
struct BaseRef {
    system: Option<StyleSystemConfig>,
}

/// A built-in theme definition (base + style files merged).
pub struct BuiltinTheme {
    pub templates: Vec<(&'static str, &'static str)>,
    pub static_files: Vec<(&'static str, &'static [u8])>,
    pub config_toml: &'static str,
}

/// Returns the list of all built-in theme names (directory names under `defaults/styles/`).
pub fn builtin_theme_names() -> Vec<&'static str> {
    let mut names: Vec<&str> = STYLES_DIR
        .dirs()
        .map(|d| d.path().file_name().unwrap().to_str().unwrap())
        .collect();
    names.sort();
    names
}

/// Returns the built-in base templates and static files for the given base name.
fn builtin_base(name: &str) -> Option<(Vec<(&'static str, &'static str)>, Vec<(&'static str, &'static [u8])>)> {
    // Collect templates
    let templates = match BASES_DIR.get_dir(&format!("{}/templates", name)) {
        Some(tpl_dir) => tpl_dir
            .files()
            .filter_map(|f| {
                let fname = f.path().file_name()?.to_str()?;
                let content = f.contents_utf8()?;
                Some((fname, content))
            })
            .collect(),
        None => vec![],
    };

    // Collect static files
    let static_files = match BASES_DIR.get_dir(&format!("{}/static", name)) {
        Some(static_dir) => collect_embedded_files(static_dir, ""),
        None => vec![],
    };

    if templates.is_empty() && static_files.is_empty() {
        return None;
    }

    Some((templates, static_files))
}

/// Returns the base name for a built-in theme by parsing its style config.toml.
pub fn base_name_for_theme(theme_name: &str) -> Option<String> {
    let config_toml = STYLES_DIR
        .get_file(&format!("{}/config.toml", theme_name))?
        .contents_utf8()?;
    let base_ref: BaseRef = toml::from_str(config_toml).ok()?;
    Some(
        base_ref
            .system
            .and_then(|s| s.base)
            .unwrap_or_else(|| DEFAULT_BASE.to_string()),
    )
}

/// Returns the built-in theme data for the given name.
/// The returned `BuiltinTheme` contains base files merged with style-specific
/// overrides: base provides the foundation, style files override matching paths.
pub fn builtin_theme(name: &str) -> Option<BuiltinTheme> {
    // Verify the style directory exists
    STYLES_DIR.get_dir(name)?;

    let config_toml = STYLES_DIR
        .get_file(&format!("{}/config.toml", name))?
        .contents_utf8()?;

    let base_name = base_name_for_theme(name).unwrap_or_else(|| DEFAULT_BASE.to_string());

    // Start with base files as the foundation
    let (mut templates, mut static_files) = builtin_base(&base_name)
        .unwrap_or_else(|| (vec![], vec![]));

    // Override with style-specific templates (if any)
    if let Some(tpl_dir) = STYLES_DIR.get_dir(&format!("{}/templates", name)) {
        for file in tpl_dir.files() {
            if let (Some(fname), Some(content)) = (
                file.path().file_name().and_then(|n| n.to_str()),
                file.contents_utf8(),
            ) {
                if let Some(pos) = templates.iter().position(|(n, _)| *n == fname) {
                    templates[pos] = (fname, content);
                } else {
                    templates.push((fname, content));
                }
            }
        }
    }

    // Override with style-specific static files (if any)
    if let Some(static_dir) = STYLES_DIR.get_dir(&format!("{}/static", name)) {
        let style_statics = collect_embedded_files(static_dir, "");
        for (path, content) in style_statics {
            if let Some(pos) = static_files.iter().position(|(p, _)| *p == path) {
                static_files[pos] = (path, content);
            } else {
                static_files.push((path, content));
            }
        }
    }

    Some(BuiltinTheme {
        templates,
        static_files,
        config_toml,
    })
}

/// Recursively collect files under an embedded Dir, building relative paths.
/// Used for both base/style static files and init scaffold pages.
///
/// Leaks path strings via `Box::leak` to produce `&'static str`. This is
/// acceptable because there is a fixed, small number of embedded files and
/// they live for the entire program lifetime.
fn collect_embedded_files(dir: &'static Dir<'static>, prefix: &str) -> Vec<(&'static str, &'static [u8])> {
    let mut result = Vec::new();

    for file in dir.files() {
        if let Some(name) = file.path().file_name().and_then(|n| n.to_str()) {
            let content = file.contents();
            if prefix.is_empty() {
                result.push((name, content));
            } else {
                let full: &'static str = Box::leak(format!("{}/{}", prefix, name).into_boxed_str());
                result.push((full, content));
            }
        }
    }

    for sub in dir.dirs() {
        let sub_name = sub.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        let new_prefix = if prefix.is_empty() {
            sub_name.to_string()
        } else {
            format!("{}/{}", prefix, sub_name)
        };
        result.extend(collect_embedded_files(sub, &new_prefix));
    }

    result
}

/// Returns all default templates as (name, source) pairs for Tera registration.
pub fn default_templates(theme_name: &str) -> Vec<(&'static str, &'static str)> {
    builtin_theme(theme_name)
        .map(|t| t.templates)
        .unwrap_or_default()
}

/// Returns all default static files as (relative_path, content) pairs.
pub fn default_static_files(theme_name: &str) -> Vec<(&'static str, &'static [u8])> {
    builtin_theme(theme_name)
        .map(|t| t.static_files)
        .unwrap_or_default()
}

/// Returns all init scaffold files for the given theme as (relative_path, content) pairs.
pub fn init_files(theme_name: &str) -> Vec<(&'static str, &'static [u8])> {
    // Light existence check — avoid full base+style merge just to validate
    if STYLES_DIR.get_dir(theme_name).is_none() {
        return vec![];
    }

    let mut files = Vec::new();

    // Top-level files (config.toml)
    if let Some(f) = DEFAULTS_DIR.get_file("config.toml") {
        files.push(("config.toml", f.contents()));
    }

    // Sample pages under defaults/pages/
    if let Some(pages_dir) = DEFAULTS_DIR.get_dir("pages") {
        collect_embedded_files_into(pages_dir, "pages", &mut files);
    }

    files
}

/// Recursively collect embedded files, pushing into an existing Vec.
/// Variant of `collect_embedded_files` that always prefixes (no bare-name case).
fn collect_embedded_files_into(
    dir: &'static Dir<'static>,
    prefix: &str,
    out: &mut Vec<(&'static str, &'static [u8])>,
) {
    for file in dir.files() {
        if let Some(name) = file.path().file_name().and_then(|n| n.to_str()) {
            let full: &'static str = Box::leak(format!("{}/{}", prefix, name).into_boxed_str());
            out.push((full, file.contents()));
        }
    }
    for sub in dir.dirs() {
        let sub_name = sub.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        let new_prefix = format!("{}/{}", prefix, sub_name);
        collect_embedded_files_into(sub, &new_prefix, out);
    }
}

/// Returns base files as (relative_path, content) pairs.
/// Paths are relative to the project root (e.g. "bases/standard/templates/base.html").
pub fn init_base_files(base_name: &str) -> Vec<(String, &'static [u8])> {
    let (templates, static_files) = match builtin_base(base_name) {
        Some(t) => t,
        None => return vec![],
    };

    let mut files: Vec<(String, &'static [u8])> = Vec::new();

    for (name, content) in templates {
        files.push((
            format!("bases/{}/templates/{}", base_name, name),
            content.as_bytes(),
        ));
    }

    for (rel_path, content) in static_files {
        files.push((
            format!("bases/{}/static/{}", base_name, rel_path),
            content,
        ));
    }

    files
}

/// Returns style-specific files as (relative_path, content) pairs.
/// Only includes files that belong to the style itself (not base files).
/// Paths are relative to the project root (e.g. "styles/default/static/css/main.css").
pub fn init_style_files(theme_name: &str) -> Vec<(String, &'static [u8])> {
    if STYLES_DIR.get_dir(theme_name).is_none() {
        return vec![];
    }

    let mut files: Vec<(String, &'static [u8])> = Vec::new();

    // config.toml
    if let Some(config_file) = STYLES_DIR.get_file(&format!("{}/config.toml", theme_name)) {
        if let Some(content) = config_file.contents_utf8() {
            files.push((
                format!("styles/{}/config.toml", theme_name),
                content.as_bytes(),
            ));
        }
    }

    // Style-specific templates (if any)
    if let Some(tpl_dir) = STYLES_DIR.get_dir(&format!("{}/templates", theme_name)) {
        for file in tpl_dir.files() {
            if let (Some(fname), Some(content)) = (
                file.path().file_name().and_then(|n| n.to_str()),
                file.contents_utf8(),
            ) {
                files.push((
                    format!("styles/{}/templates/{}", theme_name, fname),
                    content.as_bytes(),
                ));
            }
        }
    }

    // Style-specific static files (if any)
    if let Some(static_dir) = STYLES_DIR.get_dir(&format!("{}/static", theme_name)) {
        for (rel_path, content) in collect_embedded_files(static_dir, "") {
            files.push((
                format!("styles/{}/static/{}", theme_name, rel_path),
                content,
            ));
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_theme_names_not_empty() {
        let names = builtin_theme_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"default"));
        assert!(names.contains(&"monotone"));
    }

    #[test]
    fn test_builtin_theme_names_excludes_bases() {
        let names = builtin_theme_names();
        assert!(!names.contains(&"standard"));
    }

    #[test]
    fn test_builtin_theme_default_exists() {
        let theme = builtin_theme("default");
        assert!(theme.is_some());
        let theme = theme.unwrap();
        assert!(!theme.templates.is_empty());
        assert!(!theme.static_files.is_empty());
        assert!(!theme.config_toml.is_empty());
    }

    #[test]
    fn test_builtin_theme_merges_base_templates() {
        let theme = builtin_theme("default").unwrap();
        let names: Vec<&str> = theme.templates.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"base.html"));
        assert!(names.contains(&"page.html"));
        assert!(names.contains(&"portal.html"));
    }

    #[test]
    fn test_builtin_theme_merges_base_and_style_statics() {
        let theme = builtin_theme("default").unwrap();
        let paths: Vec<&str> = theme.static_files.iter().map(|(p, _)| *p).collect();
        assert!(paths.contains(&"css/main.css"));
        assert!(paths.contains(&"js/main.js"));
        assert!(paths.contains(&"favicon.svg"));
    }

    #[test]
    fn test_builtin_theme_nonexistent() {
        assert!(builtin_theme("nonexistent_theme_xyz").is_none());
    }

    #[test]
    fn test_default_templates_for_default() {
        let templates = default_templates("default");
        assert!(!templates.is_empty());
        let names: Vec<&str> = templates.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"page.html") || names.contains(&"base.html"));
    }

    #[test]
    fn test_default_static_files_for_default() {
        let files = default_static_files("default");
        assert!(!files.is_empty());
    }

    #[test]
    fn test_init_files_for_default() {
        let files = init_files("default");
        assert!(!files.is_empty());
        let paths: Vec<&str> = files.iter().map(|(p, _)| *p).collect();
        assert!(paths.contains(&"config.toml"));
    }

    #[test]
    fn test_init_files_for_nonexistent() {
        let files = init_files("nonexistent_theme_xyz");
        assert!(files.is_empty());
    }

    #[test]
    fn test_init_style_files_for_default() {
        let files = init_style_files("default");
        assert!(!files.is_empty());
        assert!(files.iter().any(|(p, _)| p.starts_with("styles/default/")));
        // Style-specific files only — no base files
        assert!(files.iter().any(|(p, _)| p.ends_with("css/main.css")));
        assert!(files.iter().any(|(p, _)| p.ends_with("config.toml")));
        assert!(!files.iter().any(|(p, _)| p.ends_with("base.html")));
        assert!(!files.iter().any(|(p, _)| p.ends_with("js/main.js")));
        assert!(!files.iter().any(|(p, _)| p.ends_with("favicon.svg")));
    }

    #[test]
    fn test_init_base_files_for_standard() {
        let files = init_base_files("standard");
        assert!(!files.is_empty());
        assert!(files.iter().all(|(p, _)| p.starts_with("bases/standard/")));
        assert!(files.iter().any(|(p, _)| p.ends_with("base.html")));
        assert!(files.iter().any(|(p, _)| p.ends_with("page.html")));
        assert!(files.iter().any(|(p, _)| p.ends_with("portal.html")));
        assert!(files.iter().any(|(p, _)| p.ends_with("js/main.js")));
        assert!(files.iter().any(|(p, _)| p.ends_with("favicon.svg")));
        // No style-specific files
        assert!(!files.iter().any(|(p, _)| p.ends_with("css/main.css")));
    }

    #[test]
    fn test_init_base_files_for_nonexistent() {
        let files = init_base_files("nonexistent_base_xyz");
        assert!(files.is_empty());
    }

    #[test]
    fn test_base_name_for_theme() {
        assert_eq!(base_name_for_theme("default").as_deref(), Some("standard"));
        assert_eq!(base_name_for_theme("monotone").as_deref(), Some("standard"));
        assert!(base_name_for_theme("nonexistent_theme_xyz").is_none());
    }

    #[test]
    fn test_monotone_theme_uses_same_base() {
        let default_theme = builtin_theme("default").unwrap();
        let monotone_theme = builtin_theme("monotone").unwrap();
        let default_tpls: Vec<&str> = default_theme.templates.iter().map(|(n, _)| *n).collect();
        let monotone_tpls: Vec<&str> = monotone_theme.templates.iter().map(|(n, _)| *n).collect();
        assert_eq!(default_tpls, monotone_tpls);
    }
}
