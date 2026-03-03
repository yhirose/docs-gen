// Default embedded theme and scaffold files.
// The entire `defaults/themes/` directory is embedded at compile time via
// `include_dir!`, so adding a new theme only requires placing files under
// `defaults/themes/<name>/` — no manual constants needed.

use include_dir::{include_dir, Dir};

/// All built-in themes, embedded at compile time.
static THEMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/defaults/themes");

/// All scaffold files (config.toml, sample pages), embedded at compile time.
static DEFAULTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/defaults");

/// A built-in theme definition.
pub struct BuiltinTheme {
    pub templates: Vec<(&'static str, &'static str)>,
    pub static_files: Vec<(&'static str, &'static str)>,
    pub config_toml: &'static str,
}

/// Returns the list of all built-in theme names (directory names under `defaults/themes/`).
pub fn builtin_theme_names() -> Vec<&'static str> {
    let mut names: Vec<&str> = THEMES_DIR
        .dirs()
        .map(|d| d.path().file_name().unwrap().to_str().unwrap())
        .collect();
    names.sort();
    names
}

/// Returns the built-in theme data for the given name.
pub fn builtin_theme(name: &str) -> Option<BuiltinTheme> {
    // Verify the theme directory exists
    THEMES_DIR.get_dir(name)?;

    // config.toml — use full path from THEMES_DIR root (include_dir 0.7 stores
    // all paths relative to the embedded root, so nested lookups must use the
    // full path even when invoked on a sub-Dir).
    let config_toml = THEMES_DIR
        .get_file(&format!("{}/config.toml", name))?
        .contents_utf8()?;

    // Collect templates (files under templates/)
    let templates_dir = THEMES_DIR.get_dir(&format!("{}/templates", name))?;
    let templates: Vec<(&str, &str)> = templates_dir
        .files()
        .filter_map(|f| {
            let name = f.path().file_name()?.to_str()?;
            let content = f.contents_utf8()?;
            Some((name, content))
        })
        .collect();

    // Collect static files (files under static/, preserving relative paths)
    let static_dir = THEMES_DIR.get_dir(&format!("{}/static", name))?;
    let static_files = collect_static_files(static_dir, "");

    Some(BuiltinTheme {
        templates,
        static_files,
        config_toml,
    })
}

/// Recursively collect files under a Dir, building relative paths.
fn collect_static_files(dir: &'static Dir<'static>, prefix: &str) -> Vec<(&'static str, &'static str)> {
    let mut result = Vec::new();

    for file in dir.files() {
        if let (Some(name), Some(content)) = (
            file.path().file_name().and_then(|n| n.to_str()),
            file.contents_utf8(),
        ) {
            if prefix.is_empty() {
                result.push((name, content));
            } else {
                // Leak the owned String to get a &'static str.
                // This is acceptable because there is a fixed, small number of
                // built-in theme files and they live for the entire program lifetime.
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
        result.extend(collect_static_files(sub, &new_prefix));
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
pub fn default_static_files(theme_name: &str) -> Vec<(&'static str, &'static str)> {
    builtin_theme(theme_name)
        .map(|t| t.static_files)
        .unwrap_or_default()
}

/// Returns all init scaffold files for the given theme as (relative_path, content) pairs.
pub fn init_files(theme_name: &str) -> Vec<(&'static str, &'static str)> {
    if builtin_theme(theme_name).is_none() {
        return vec![];
    }

    let mut files = Vec::new();

    // Top-level files (config.toml)
    if let Some(f) = DEFAULTS_DIR.get_file("config.toml") {
        if let Some(content) = f.contents_utf8() {
            files.push(("config.toml", content));
        }
    }

    // Sample pages under defaults/pages/
    if let Some(pages_dir) = DEFAULTS_DIR.get_dir("pages") {
        collect_init_pages(pages_dir, "pages", &mut files);
    }

    files
}

/// Recursively collect page files for init scaffolding.
fn collect_init_pages(
    dir: &'static Dir<'static>,
    prefix: &str,
    out: &mut Vec<(&'static str, &'static str)>,
) {
    for file in dir.files() {
        if let (Some(name), Some(content)) = (
            file.path().file_name().and_then(|n| n.to_str()),
            file.contents_utf8(),
        ) {
            let full: &'static str = Box::leak(format!("{}/{}", prefix, name).into_boxed_str());
            out.push((full, content));
        }
    }
    for sub in dir.dirs() {
        let sub_name = sub.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
        let new_prefix = format!("{}/{}", prefix, sub_name);
        collect_init_pages(sub, &new_prefix, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_theme_names_not_empty() {
        let names = builtin_theme_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"default"));
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
    fn test_builtin_theme_nonexistent() {
        assert!(builtin_theme("nonexistent_theme_xyz").is_none());
    }

    #[test]
    fn test_default_templates_for_default() {
        let templates = default_templates("default");
        assert!(!templates.is_empty());
        // Should have base.html or page.html
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
    fn test_init_theme_files_for_default() {
        let files = init_theme_files("default");
        assert!(!files.is_empty());
        // Should contain paths like "themes/default/..."
        assert!(files.iter().any(|(p, _)| p.starts_with("themes/default/")));
    }
}

/// Returns theme files for init as (relative_path, content) pairs.
/// Paths are relative to the project root (e.g. "themes/default/templates/base.html").
pub fn init_theme_files(theme_name: &str) -> Vec<(String, &'static str)> {
    let theme = match builtin_theme(theme_name) {
        Some(t) => t,
        None => return vec![],
    };

    let mut files: Vec<(String, &'static str)> = Vec::new();

    // Theme config.toml
    files.push((
        format!("themes/{}/config.toml", theme_name),
        theme.config_toml,
    ));

    // Templates
    for (name, content) in &theme.templates {
        files.push((
            format!("themes/{}/templates/{}", theme_name, name),
            content,
        ));
    }

    // Static files
    for (rel_path, content) in &theme.static_files {
        files.push((
            format!("themes/{}/static/{}", theme_name, rel_path),
            content,
        ));
    }

    files
}
