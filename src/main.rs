mod builder;
mod check;
mod config;
mod defaults;
mod markdown;
mod serve;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand, CommandFactory};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, about = "A simple static site generator")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Build the documentation site
    Build {
        /// Source directory containing config.toml
        src: PathBuf,
        /// Output directory
        out: PathBuf,
        /// Theme to use (overrides config.toml)
        #[arg(long)]
        theme: Option<String>,
    },
    /// Initialize a new docs project with default scaffold files
    Init {
        /// Target directory to initialize (default: current directory)
        #[arg(default_value = ".")]
        src: PathBuf,
        /// Theme to install (default: "default")
        #[arg(long, default_value = "default")]
        theme: String,
    },
    /// Start a local development server with live-reload
    Serve {
        /// Source directory containing config.toml
        #[arg(default_value = ".")]
        src: PathBuf,

        /// Port number for the HTTP server
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Open browser automatically
        #[arg(long)]
        open: bool,

        /// Theme to use (overrides config.toml)
        #[arg(long)]
        theme: Option<String>,
    },
    /// Check the site for errors (broken links, order issues, etc.)
    Check {
        /// Source directory containing config.toml
        #[arg(default_value = ".")]
        src: PathBuf,
        /// Automatically fix safe issues (e.g. missing ../ in image paths)
        #[arg(long)]
        fix: bool,
    },
    /// Manage themes
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
}

#[derive(Subcommand)]
enum ThemeAction {
    /// List available built-in themes
    List,
    /// Install a built-in theme into the project
    Install {
        /// Theme name to install
        name: String,
        /// Target project directory
        #[arg(default_value = ".")]
        src: PathBuf,
        /// Force overwrite without confirmation
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Build { src, out, theme }) => {
            builder::build(&src, &out, theme.as_deref())
        }
        Some(Command::Init { src, theme }) => cmd_init(&src, &theme),
        Some(Command::Serve { src, port, open, theme }) => {
            serve::serve(&src, port, open, theme.as_deref())
        }
        Some(Command::Check { src, fix }) => {
            match check::run(&src, fix) {
                Ok(has_errors) => {
                    if has_errors {
                        std::process::exit(1);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {:#}", e);
                    std::process::exit(2);
                }
            }
        }
        Some(Command::Theme { action }) => match action {
            ThemeAction::List => cmd_theme_list(),
            ThemeAction::Install { name, src, force } => cmd_theme_install(&name, &src, force),
        },
        None => {
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}

fn cmd_init(target: &Path, theme_name: &str) -> Result<()> {
    // Validate theme exists
    if defaults::builtin_theme(theme_name).is_none() {
        anyhow::bail!(
            "Unknown theme '{}'. Available themes: {}",
            theme_name,
            defaults::builtin_theme_names().join(", ")
        );
    }

    let mut skipped = 0usize;
    let mut created = 0usize;

    // Write site-level init files (config.toml, pages) only — no theme files.
    // Theme files can be installed separately with `docs-gen theme install`.
    for (rel_path, content) in defaults::init_files(theme_name) {
        let dest = target.join(rel_path);
        if dest.exists() {
            eprintln!("Skipping (already exists): {}", dest.display());
            skipped += 1;
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        // For config.toml, substitute the theme name
        if rel_path == "config.toml" {
            let text = std::str::from_utf8(content)?;
            let patched = text.replace(
                "theme = \"default\"",
                &format!("theme = \"{}\"", theme_name),
            );
            fs::write(&dest, patched)?;
        } else {
            fs::write(&dest, content)?;
        }
        println!("Created: {}", dest.display());
        created += 1;
    }

    println!("\nInit complete: {} file(s) created, {} skipped.", created, skipped);
    println!(
        "\nTo customize the theme, run: docs-gen theme install {}",
        theme_name
    );
    Ok(())
}

fn cmd_theme_list() -> Result<()> {
    println!("Available built-in themes:");
    for name in defaults::builtin_theme_names() {
        println!("  {}", name);
    }
    Ok(())
}

fn cmd_theme_install(theme_name: &str, target: &Path, force: bool) -> Result<()> {
    // Validate theme exists
    if defaults::builtin_theme(theme_name).is_none() {
        anyhow::bail!(
            "Unknown theme '{}'. Available themes: {}",
            theme_name,
            defaults::builtin_theme_names().join(", ")
        );
    }

    let theme_dir = target.join("themes").join(theme_name);

    // Check if theme already exists
    if theme_dir.exists() && !force {
        print!(
            "Theme '{}' already exists at {}. Overwrite? [y/N] ",
            theme_name,
            theme_dir.display()
        );
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") && !answer.trim().eq_ignore_ascii_case("yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let mut created = 0usize;

    for (rel_path, content) in defaults::init_theme_files(theme_name) {
        let dest = target.join(&rel_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, content)?;
        println!("Installed: {}", dest.display());
        created += 1;
    }

    // Record the docs-gen version used to install this theme.
    let config_path = target.join("themes").join(theme_name).join("config.toml");
    if config_path.exists() {
        let mut config_content = fs::read_to_string(&config_path)?;
        config_content.push_str(&format!(
            "\n[meta]\ndocs-gen-version = \"{}\"\n",
            env!("CARGO_PKG_VERSION")
        ));
        fs::write(&config_path, config_content)?;
    }

    println!(
        "\nTheme '{}' installed: {} file(s).",
        theme_name, created
    );
    Ok(())
}

