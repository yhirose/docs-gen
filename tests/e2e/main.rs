mod helpers;

use helpers::{base_url, create_driver, setup};
use thirtyfour::prelude::*;
use std::time::Duration;

// ─── Helper: ensure driver.quit() is always called ───────────
/// Run an async test body, ensuring `driver.quit()` is called even on failure.
async fn with_driver<F, Fut>(f: F) -> WebDriverResult<()>
where
    F: FnOnce(WebDriver) -> Fut,
    Fut: std::future::Future<Output = WebDriverResult<()>>,
{
    setup();
    let driver = create_driver().await?;
    let result = f(driver.clone()).await;
    let _ = driver.quit().await;
    result
}

// ═══════════════════════════════════════════════════════════════
//  1. Page loading
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_homepage_loads() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let title = driver.title().await?;
        assert!(
            title.contains("docs-gen"),
            "Page title should contain 'docs-gen', got: {}",
            title
        );

        let content = driver.find(By::Css("main.content")).await?;
        assert!(content.is_displayed().await?);

        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore]
async fn e2e_guide_index_loads() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/users-guide/", base_url())).await?;

        let title = driver.title().await?;
        assert!(
            title.contains("Guide"),
            "Guide page title should contain 'Guide', got: {}",
            title
        );

        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore]
async fn e2e_guide_subpage_loads() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver
            .goto(&format!("{}/users-guide/01-getting-started/", base_url()))
            .await?;

        let title = driver.title().await?;
        assert!(
            title.contains("Getting Started"),
            "Subpage title should contain 'Getting Started', got: {}",
            title
        );

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  2. Navigation links (header)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_header_nav_links() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let nav_links = driver.find_all(By::Css(".header-nav a")).await?;
        assert!(
            nav_links.len() >= 3,
            "Expected at least 3 nav links (Home + User's Guide + Developer's Guide), got {}",
            nav_links.len()
        );

        for link in &nav_links {
            let text = link.text().await?;
            if text.contains("User") && text.contains("Guide") {
                link.click().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
                let url = driver.current_url().await?;
                assert!(
                    url.as_str().contains("/users-guide"),
                    "After clicking User's Guide, URL should contain /users-guide, got: {}",
                    url
                );
                break;
            }
        }

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  3. Sidebar
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_sidebar_displayed_on_guide() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/users-guide/", base_url())).await?;

        let sidebar = driver.find(By::Css("aside.sidebar")).await?;
        assert!(sidebar.is_displayed().await?, "Sidebar should be visible on guide pages");

        let sidebar_links = driver.find_all(By::Css(".sidebar-nav a")).await?;
        assert!(
            !sidebar_links.is_empty(),
            "Sidebar should contain navigation links"
        );

        let mut found_getting_started = false;
        let mut found_writing_pages = false;
        let mut found_configuration = false;

        for link in &sidebar_links {
            let text = link.text().await?;
            if text.contains("Getting Started") {
                found_getting_started = true;
            }
            if text.contains("Writing Pages") {
                found_writing_pages = true;
            }
            if text.contains("Configuration") {
                found_configuration = true;
            }
        }

        assert!(found_getting_started, "Sidebar should have 'Getting Started' link");
        assert!(found_writing_pages, "Sidebar should have 'Writing Pages' link");
        assert!(found_configuration, "Sidebar should have 'Configuration' link");

        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore]
async fn e2e_sidebar_not_on_homepage() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let layout = driver.find(By::Css(".layout")).await?;
        let class = layout.attr("class").await?.unwrap_or_default();
        assert!(
            class.contains("no-sidebar"),
            "Homepage layout should have 'no-sidebar' class, got: {}",
            class
        );

        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore]
async fn e2e_sidebar_link_navigation() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/users-guide/", base_url())).await?;

        let link = driver.find(By::LinkText("Getting Started")).await?;
        link.click().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let url = driver.current_url().await?;
        assert!(
            url.as_str().contains("01-getting-started"),
            "After clicking 'Getting Started', URL should contain '01-getting-started', got: {}",
            url
        );

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  4. Search
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_search_modal_opens() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let search_btn = driver.find(By::Css(".search-btn")).await?;
        search_btn.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let overlay = driver.find(By::Id("search-overlay")).await?;
        let class = overlay.attr("class").await?.unwrap_or_default();
        assert!(
            class.contains("active") || overlay.is_displayed().await?,
            "Search overlay should be active/visible after clicking search button"
        );

        let input = driver.find(By::Id("search-input")).await?;
        assert!(input.is_displayed().await?, "Search input should be displayed");

        Ok(())
    })
    .await
}

#[tokio::test]
#[ignore]
async fn e2e_search_returns_results() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let search_btn = driver.find(By::Css(".search-btn")).await?;
        search_btn.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let input = driver.find(By::Id("search-input")).await?;
        input.send_keys("getting started").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let results = driver.find_all(By::Css("#search-results li")).await?;
        assert!(
            !results.is_empty(),
            "Search for 'getting started' should return results"
        );

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  5. Dark / Light theme toggle
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_theme_toggle() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let html = driver.find(By::Css("html")).await?;
        let initial_theme = html.attr("data-theme").await?;

        let toggle = driver.find(By::Css(".theme-toggle")).await?;
        toggle.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let new_theme = html.attr("data-theme").await?;
        assert_ne!(
            initial_theme, new_theme,
            "Theme should change after clicking toggle. Initial: {:?}, New: {:?}",
            initial_theme, new_theme
        );

        toggle.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let restored_theme = html.attr("data-theme").await?;
        assert_eq!(
            initial_theme, restored_theme,
            "Theme should restore after double toggle. Initial: {:?}, Restored: {:?}",
            initial_theme, restored_theme
        );

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  6. Guide page navigation (prev/next)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_guide_page_sequence() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        let pages = [
            ("01-getting-started", "Getting Started"),
            ("02-writing-pages", "Writing Pages"),
            ("03-configuration", "Configuration"),
        ];

        for (slug, expected_title) in &pages {
            driver
                .goto(&format!("{}/users-guide/{}/", base_url(), slug))
                .await?;

            let title = driver.title().await?;
            assert!(
                title.contains(expected_title),
                "Page {} title should contain '{}', got: {}",
                slug,
                expected_title,
                title
            );

            let article = driver.find(By::Css("article")).await?;
            let text = article.text().await?;
            assert!(
                !text.is_empty(),
                "Guide page {} article should have content",
                slug
            );
        }

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  7. Internal link validation
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_internal_links_valid() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        let pages_to_check = [
            format!("{}/", base_url()),
            format!("{}/users-guide/", base_url()),
            format!("{}/users-guide/01-getting-started/", base_url()),
        ];

        let server_origin = format!("http://localhost:{}", 8123);

        for page_url in &pages_to_check {
            driver.goto(page_url).await?;

            // Collect hrefs first (as Strings) to avoid stale element references
            // after navigation.
            let links = driver.find_all(By::Css("a[href]")).await?;
            let mut hrefs: Vec<String> = Vec::new();
            for link in &links {
                if let Some(h) = link.attr("href").await? {
                    hrefs.push(h);
                }
            }

            for href in &hrefs {
                if href.starts_with("http") && !href.starts_with(&server_origin) {
                    continue;
                }
                if href.starts_with('#') || href.starts_with("javascript:") {
                    continue;
                }

                let full_url = if href.starts_with('/') {
                    format!("{}{}", server_origin, href)
                } else if href.starts_with("http") {
                    href.clone()
                } else {
                    let base = page_url.trim_end_matches('/');
                    format!("{}/{}", base, href)
                };

                driver.goto(&full_url).await?;
                let body = driver.find(By::Css("body")).await?;
                let text = body.text().await?;
                assert!(
                    !text.contains("404 Not Found"),
                    "Broken internal link: {} (from {})",
                    href,
                    page_url
                );
            }
        }

        Ok(())
    })
    .await
}

// ═══════════════════════════════════════════════════════════════
//  8. Footer
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn e2e_footer_present() -> WebDriverResult<()> {
    with_driver(|driver| async move {
        driver.goto(&format!("{}/", base_url())).await?;

        let footer = driver.find(By::Css("footer.footer")).await?;
        assert!(footer.is_displayed().await?, "Footer should be visible");

        let footer_text = footer.text().await?;
        assert!(
            footer_text.contains("2026"),
            "Footer should contain copyright text, got: {}",
            footer_text
        );

        Ok(())
    })
    .await
}
