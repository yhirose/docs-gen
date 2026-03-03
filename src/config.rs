use crate::defaults;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct SiteConfig {
    pub system: System,
    pub site: Site,
    /// Theme-level highlight settings (loaded from theme config, not site config).
    #[serde(skip)]
    pub highlight: Option<Highlight>,
    #[serde(default)]
    pub nav: Vec<NavLink>,
}

fn default_theme_name() -> String {
    "default".to_string()
}

/// Theme-specific configuration loaded from `themes/<name>/config.toml`.
#[derive(Debug, Deserialize, Default)]
pub struct ThemeConfig {
    pub highlight: Option<Highlight>,
}

/// A navigation link entry defined in config.toml under [[nav]].
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NavLink {
    pub label: String,
    /// Absolute or external URL (e.g. GitHub link).
    pub url: Option<String>,
    /// Path relative to /<base_path>/<lang>/ (e.g. "tour/").
    pub path: Option<String>,
    /// Optional inline SVG string to display as an icon.
    pub icon_svg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Site {
    pub title: String,
    pub version: Option<String>,
    /// Optional hostname (e.g. "https://example.github.io"). Combined with
    /// base_path to form the full base URL.
    pub hostname: Option<String>,
    #[serde(default)]
    pub base_path: String,
    /// Optional footer message displayed at the bottom of every page.
    pub footer_message: Option<String>,
}

impl Site {
    /// Returns the full base URL derived from hostname + base_path.
    /// Falls back to base_path alone if hostname is not set.
    pub fn base_url(&self) -> String {
        match &self.hostname {
            Some(h) => format!("{}{}", h.trim_end_matches('/'), self.base_path),
            None => self.base_path.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct System {
    #[serde(default = "default_theme_name")]
    pub theme: String,
    pub langs: Vec<String>,
}

impl System {
    /// Returns the default language, which is the first entry in langs.
    pub fn default_lang(&self) -> &str {
        self.langs.first().map(|s| s.as_str()).unwrap_or("en")
    }

    /// Returns `true` when only a single language is configured.
    pub fn is_single_lang(&self) -> bool {
        self.langs.len() == 1
    }
}

#[derive(Debug, Deserialize)]
pub struct Highlight {
    pub dark_theme: Option<String>,
    pub light_theme: Option<String>,
}

impl SiteConfig {
    /// Load site config from `<src_dir>/config.toml`, then merge theme config.
    /// If `theme_override` is Some, it takes precedence over the `theme` field
    /// in config.toml.
    pub fn load(src_dir: &Path, theme_override: Option<&str>) -> Result<Self> {
        let path = src_dir.join("config.toml");
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let mut config: SiteConfig =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;

        // --theme overrides config.toml
        if let Some(t) = theme_override {
            config.system.theme = t.to_string();
        }

        // Validate required fields
        if config.system.langs.is_empty() {
            anyhow::bail!("[system] langs must not be empty. Please specify at least one language.");
        }

        // Normalize base_path: strip trailing slash (but keep empty for root)
        let bp = config.site.base_path.trim_end_matches('/').to_string();
        config.site.base_path = bp;

        // Load theme config
        let theme_config = Self::load_theme_config(src_dir, &config.system.theme)?;
        config.highlight = theme_config.highlight;

        Ok(config)
    }

    /// Load theme-specific config.toml. Tries:
    /// 1. `<src_dir>/themes/<theme>/config.toml` (user project)
    /// 2. Built-in theme defaults
    fn load_theme_config(src_dir: &Path, theme_name: &str) -> Result<ThemeConfig> {
        let theme_config_path = src_dir.join("themes").join(theme_name).join("config.toml");
        if theme_config_path.exists() {
            let content = std::fs::read_to_string(&theme_config_path)
                .with_context(|| format!("Failed to read {}", theme_config_path.display()))?;
            let tc: ThemeConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {}", theme_config_path.display()))?;
            return Ok(tc);
        }

        // Fall back to built-in theme
        if let Some(builtin) = defaults::builtin_theme(theme_name) {
            let tc: ThemeConfig = toml::from_str(builtin.config_toml)
                .with_context(|| format!("Failed to parse built-in theme config for '{}'", theme_name))?;
            return Ok(tc);
        }

        // Unknown theme: return empty config (no highlight settings)
        eprintln!("Warning: theme '{}' not found, using defaults", theme_name);
        Ok(ThemeConfig::default())
    }

    pub fn highlight_dark_theme(&self) -> &str {
        self.highlight
            .as_ref()
            .and_then(|h| h.dark_theme.as_deref())
            .unwrap_or("base16-ocean.dark")
    }

    pub fn highlight_light_theme(&self) -> Option<&str> {
        self.highlight
            .as_ref()
            .and_then(|h| h.light_theme.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_base_url_with_hostname() {
        let site = Site {
            title: "Test".into(),
            version: None,
            hostname: Some("https://example.com".into()),
            base_path: "/docs".into(),
            footer_message: None,
        };
        assert_eq!(site.base_url(), "https://example.com/docs");
    }

    #[test]
    fn test_base_url_with_hostname_trailing_slash() {
        let site = Site {
            title: "Test".into(),
            version: None,
            hostname: Some("https://example.com/".into()),
            base_path: "/docs".into(),
            footer_message: None,
        };
        assert_eq!(site.base_url(), "https://example.com/docs");
    }

    #[test]
    fn test_base_url_without_hostname() {
        let site = Site {
            title: "Test".into(),
            version: None,
            hostname: None,
            base_path: "/my-site".into(),
            footer_message: None,
        };
        assert_eq!(site.base_url(), "/my-site");
    }

    #[test]
    fn test_default_lang() {
        let system = System {
            theme: "default".into(),
            langs: vec!["ja".into(), "en".into()],
        };
        assert_eq!(system.default_lang(), "ja");
    }

    #[test]
    fn test_default_lang_fallback() {
        let system = System {
            theme: "default".into(),
            langs: vec![],
        };
        assert_eq!(system.default_lang(), "en");
    }

    #[test]
    fn test_is_single_lang() {
        let single = System { theme: "default".into(), langs: vec!["en".into()] };
        assert!(single.is_single_lang());

        let multi = System { theme: "default".into(), langs: vec!["en".into(), "ja".into()] };
        assert!(!multi.is_single_lang());
    }

    #[test]
    fn test_load_valid_config() {
        let dir = tempdir().unwrap();
        let config_content = r#"
[system]
theme = "default"
langs = ["en"]

[site]
title = "Test Site"
base_path = "/docs"
"#;
        fs::write(dir.path().join("config.toml"), config_content).unwrap();

        let config = SiteConfig::load(dir.path(), None).unwrap();
        assert_eq!(config.site.title, "Test Site");
        assert_eq!(config.site.base_path, "/docs");
        assert_eq!(config.system.theme, "default");
    }

    #[test]
    fn test_load_config_theme_override() {
        let dir = tempdir().unwrap();
        let config_content = r#"
[system]
theme = "default"
langs = ["en"]

[site]
title = "Test Site"
base_path = ""
"#;
        fs::write(dir.path().join("config.toml"), config_content).unwrap();

        let config = SiteConfig::load(dir.path(), Some("monotone")).unwrap();
        assert_eq!(config.system.theme, "monotone");
    }

    #[test]
    fn test_load_config_empty_langs() {
        let dir = tempdir().unwrap();
        let config_content = r#"
[system]
theme = "default"
langs = []

[site]
title = "Test Site"
base_path = ""
"#;
        fs::write(dir.path().join("config.toml"), config_content).unwrap();

        let result = SiteConfig::load(dir.path(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_base_path_trailing_slash_stripped() {
        let dir = tempdir().unwrap();
        let config_content = r#"
[system]
theme = "default"
langs = ["en"]

[site]
title = "Test"
base_path = "/docs/"
"#;
        fs::write(dir.path().join("config.toml"), config_content).unwrap();

        let config = SiteConfig::load(dir.path(), None).unwrap();
        assert_eq!(config.site.base_path, "/docs");
    }

    #[test]
    fn test_highlight_dark_theme_default() {
        let config = SiteConfig {
            system: System { theme: "default".into(), langs: vec!["en".into()] },
            site: Site {
                title: "T".into(),
                version: None,
                hostname: None,
                base_path: String::new(),
                footer_message: None,
            },
            highlight: None,
            nav: vec![],
        };
        assert_eq!(config.highlight_dark_theme(), "base16-ocean.dark");
    }
}
