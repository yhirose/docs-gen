use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use std::thread;
use thirtyfour::prelude::*;

const SERVER_PORT: u16 = 8123;
const GECKODRIVER_PORT: u16 = 4444;
const BASE_PATH: &str = "/docs-gen";

// ─── Process management ──────────────────────────────────────

static SERVER: OnceLock<Mutex<Child>> = OnceLock::new();
static GECKODRIVER: OnceLock<Mutex<Child>> = OnceLock::new();

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // binary name
    path.pop(); // deps
    path.push("docs-gen");
    path
}

/// Ensure geckodriver is running on GECKODRIVER_PORT.
fn ensure_geckodriver_running() {
    GECKODRIVER.get_or_init(|| {
        // Kill any stale geckodriver
        let _ = Command::new("pkill")
            .args(["-f", &format!("geckodriver.*--port {}", GECKODRIVER_PORT)])
            .status();
        thread::sleep(Duration::from_millis(500));

        let child = Command::new("geckodriver")
            .args(["--port", &GECKODRIVER_PORT.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start geckodriver. Install with: brew install geckodriver");

        wait_for_port(GECKODRIVER_PORT, Duration::from_secs(10));
        Mutex::new(child)
    });
}

/// Ensure the docs-gen serve process is running on SERVER_PORT.
fn ensure_server_running() {
    SERVER.get_or_init(|| {
        // Kill any stale serve processes on this port
        let _ = Command::new("pkill")
            .args(["-f", &format!("docs-gen serve.*--port {}", SERVER_PORT)])
            .status();
        thread::sleep(Duration::from_millis(500));

        // Build the main binary
        let status = Command::new("cargo")
            .args(["build"])
            .current_dir(project_root())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to run cargo build");
        assert!(status.success(), "cargo build failed");

        // Start docs-gen serve
        let child = Command::new(binary_path())
            .args([
                "serve",
                "docs-src",
                "--port",
                &SERVER_PORT.to_string(),
            ])
            .current_dir(project_root())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start docs-gen serve");

        wait_for_port(SERVER_PORT, Duration::from_secs(30));
        Mutex::new(child)
    });
}

fn wait_for_port(port: u16, timeout: Duration) {
    let start = Instant::now();
    loop {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return;
        }
        if start.elapsed() > timeout {
            panic!(
                "Server on port {} did not start within {:?}",
                port, timeout
            );
        }
        thread::sleep(Duration::from_millis(300));
    }
}

// ─── Public API ──────────────────────────────────────────────

/// Full base URL for the demo site.
pub fn base_url() -> String {
    format!("http://localhost:{}{}", SERVER_PORT, BASE_PATH)
}

/// Setup: ensure geckodriver + docs-gen serve are running.
pub fn setup() {
    ensure_geckodriver_running();
    ensure_server_running();
}

/// Create a new headless Firefox WebDriver session.
/// Handles stale sessions by restarting geckodriver if needed.
pub async fn create_driver() -> WebDriverResult<WebDriver> {
    let url = format!("http://localhost:{}", GECKODRIVER_PORT);
    let mut caps = DesiredCapabilities::firefox();
    caps.add_arg("-headless")?;

    match WebDriver::new(&url, caps.clone()).await {
        Ok(driver) => {
            driver
                .set_implicit_wait_timeout(Duration::from_secs(5))
                .await?;
            Ok(driver)
        }
        Err(_) => {
            // Stale session: restart geckodriver and retry
            restart_geckodriver();
            let driver = WebDriver::new(&url, caps).await?;
            driver
                .set_implicit_wait_timeout(Duration::from_secs(5))
                .await?;
            Ok(driver)
        }
    }
}

/// Restart geckodriver to clear stale sessions.
fn restart_geckodriver() {
    if let Some(guard) = GECKODRIVER.get() {
        if let Ok(mut child) = guard.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    thread::sleep(Duration::from_millis(500));

    let child = Command::new("geckodriver")
        .args(["--port", &GECKODRIVER_PORT.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to restart geckodriver");

    // Update the mutable child handle
    if let Some(guard) = GECKODRIVER.get() {
        if let Ok(mut old) = guard.lock() {
            *old = child;
        }
    }

    wait_for_port(GECKODRIVER_PORT, Duration::from_secs(10));
}
