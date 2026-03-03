use anyhow::{Context, Result};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    #[serde(default)]
    pub order: i32,
    pub status: Option<String>,
}

pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    dark_theme: String,
    light_theme: Option<String>,
}

impl MarkdownRenderer {
    pub fn new(dark_theme: &str, light_theme: Option<&str>) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            dark_theme: dark_theme.to_string(),
            light_theme: light_theme.map(|s| s.to_string()),
        }
    }

    pub fn parse_frontmatter(content: &str) -> Result<(Frontmatter, &str)> {
        let content = content.trim_start();
        if !content.starts_with("---") {
            anyhow::bail!("Missing frontmatter delimiter");
        }
        let after_first = &content[3..];
        let end = after_first
            .find("\n---")
            .context("Missing closing frontmatter delimiter")?;
        let yaml = &after_first[..end];
        let body = &after_first[end + 4..];
        let fm: Frontmatter =
            serde_yaml::from_str(yaml).context("Failed to parse frontmatter YAML")?;
        Ok((fm, body))
    }

    pub fn render(&self, markdown: &str) -> String {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS;

        let parser = Parser::new_ext(markdown, options);

        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_buf = String::new();
        let mut events: Vec<Event> = Vec::new();

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_buf.clear();
                    code_lang = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    };
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    let html = self.highlight_code(&code_buf, &code_lang);
                    events.push(Event::Html(html.into()));
                }
                Event::Text(text) if in_code_block => {
                    code_buf.push_str(&text);
                }
                other => events.push(other),
            }
        }

        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events.into_iter());
        html_output
    }

    fn highlight_code(&self, code: &str, lang: &str) -> String {
        let syntax = if lang.is_empty() {
            self.syntax_set.find_syntax_plain_text()
        } else {
            self.syntax_set
                .find_syntax_by_token(lang)
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
        };

        let dark_html = self.highlight_with_theme(code, syntax, &self.dark_theme);

        match &self.light_theme {
            Some(light) => {
                let light_html = self.highlight_with_theme(code, syntax, light);
                format!(
                    concat!(
                        "<div class=\"code-block-wrapper\">",
                        "<div data-code-theme=\"dark\">{}</div>",
                        "<div data-code-theme=\"light\">{}</div>",
                        "</div>",
                    ),
                    dark_html, light_html
                )
            }
            None => dark_html,
        }
    }

    fn highlight_with_theme(
        &self,
        code: &str,
        syntax: &syntect::parsing::SyntaxReference,
        theme_name: &str,
    ) -> String {
        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .values()
                    .next()
                    .expect("No themes available")
            });

        match highlighted_html_for_string(code, &self.syntax_set, syntax, theme) {
            Ok(html) => html,
            Err(_) => format!("<pre><code>{}</code></pre>", escape_html(code)),
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = "---\ntitle: Hello\norder: 1\n---\nBody text";
        let (fm, body) = MarkdownRenderer::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Hello");
        assert_eq!(fm.order, 1);
        assert!(body.contains("Body text"));
    }

    #[test]
    fn test_parse_frontmatter_no_order() {
        let content = "---\ntitle: Test\n---\nContent";
        let (fm, _body) = MarkdownRenderer::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test");
        assert_eq!(fm.order, 0); // default
    }

    #[test]
    fn test_parse_frontmatter_with_status() {
        let content = "---\ntitle: Draft\nstatus: draft\n---\nBody";
        let (fm, _) = MarkdownRenderer::parse_frontmatter(content).unwrap();
        assert_eq!(fm.status.as_deref(), Some("draft"));
    }

    #[test]
    fn test_parse_frontmatter_missing_delimiter() {
        let content = "No frontmatter here";
        assert!(MarkdownRenderer::parse_frontmatter(content).is_err());
    }

    #[test]
    fn test_parse_frontmatter_no_closing() {
        let content = "---\ntitle: Broken\nNo closing delimiter";
        assert!(MarkdownRenderer::parse_frontmatter(content).is_err());
    }

    #[test]
    fn test_render_paragraph() {
        let renderer = MarkdownRenderer::new("base16-ocean.dark", None);
        let html = renderer.render("Hello **world**");
        assert!(html.contains("<strong>world</strong>"));
    }

    #[test]
    fn test_render_heading() {
        let renderer = MarkdownRenderer::new("base16-ocean.dark", None);
        let html = renderer.render("# Title");
        assert!(html.contains("<h1>"));
        assert!(html.contains("Title"));
    }

    #[test]
    fn test_render_code_block() {
        let renderer = MarkdownRenderer::new("base16-ocean.dark", None);
        let html = renderer.render("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre"));
        assert!(html.contains("main"));
    }

    #[test]
    fn test_render_link() {
        let renderer = MarkdownRenderer::new("base16-ocean.dark", None);
        let html = renderer.render("[click](http://example.com)");
        assert!(html.contains("href=\"http://example.com\""));
    }

    #[test]
    fn test_render_table() {
        let renderer = MarkdownRenderer::new("base16-ocean.dark", None);
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = renderer.render(md);
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_escape_html_entities() {
        assert_eq!(escape_html("<div>&"), "&lt;div&gt;&amp;");
    }

    #[test]
    fn test_render_dual_theme_code() {
        let renderer = MarkdownRenderer::new("base16-ocean.dark", Some("InspiredGitHub"));
        let html = renderer.render("```\nhello\n```");
        assert!(html.contains("data-code-theme=\"dark\""));
        assert!(html.contains("data-code-theme=\"light\""));
    }
}
