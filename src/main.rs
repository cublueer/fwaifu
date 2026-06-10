mod api;
mod auth;
mod cache;
mod cli;
mod config;
mod daemon;
mod downloader;
mod image;
mod runner;
mod watch;
mod i18n;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

use cli::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell as ClapShell};

// ─── Helpers ─────────────────────────────────────────────────────────

fn print_help() {
    print!("{}", i18n::t("help.text"));
}

fn command_exists(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check whether a process with the given PID is still running
/// by looking for its entry in the /proc filesystem (Linux only).
fn pid_is_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// Read a PID from the first line of a lock file.
/// Returns None if the file is unreadable, empty, or contains an invalid PID.
fn read_lock_pid(lock_path: &PathBuf) -> Option<u32> {
    std::fs::read_to_string(lock_path)
        .ok()
        .and_then(|content| {
            content
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .parse::<u32>()
                .ok()
        })
}

/// Create a lock file and write the current PID to it.
fn try_create_lock_file(lock_path: &PathBuf, pid: u32) -> Result<std::fs::File, std::io::Error> {
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(lock_path)?;
    let _ = std::fs::write(lock_path, pid.to_string());
    Ok(file)
}

/// Attempt to acquire a file-based lock with PID-based stale lock detection.
///
/// Flow:
/// 1. Fast path: create a new lock file (normal case, no contention).
/// 2. If the file already exists, read the PID from it.
/// 3. Check if that PID is still alive via `/proc/{pid}`.
/// 4. If dead → stale lock → remove old file and retry.
/// 5. If alive → legitimate lock held by another process → return None.
fn try_acquire_lock(nsfw: bool) -> Option<(std::fs::File, PathBuf)> {
    let tag = if nsfw { "nsfw" } else { "sfw" };
    let lock_path = PathBuf::from(format!("/tmp/fwaifu_{tag}.lock"));
    let current_pid = std::process::id();

    // Fast path: no existing lock
    if let Ok(file) = try_create_lock_file(&lock_path, current_pid) {
        return Some((file, lock_path));
    }

    // Lock file exists — determine if it's stale
    let pid_from_file = read_lock_pid(&lock_path);
    let is_stale = pid_from_file.is_none_or(|pid| !pid_is_alive(pid));

    if !is_stale {
        return None;
    }

    // Stale lock — remove old file and retry
    let _ = std::fs::remove_file(&lock_path);
    try_create_lock_file(&lock_path, current_pid)
        .map(|file| (file, lock_path))
        .ok()
}

// ─── Background Fill ────────────────────────────────────────────────

/// Downloads a batch of images in the background to replenish stock.
/// Uses a file lock to prevent concurrent fills from running simultaneously.
async fn background_fill(
    nsfw: bool,
    config: config::Config,
    cache: Arc<cache::CacheManager>,
    api_client: Arc<api::NekosMoeClient>,
    direct_client: reqwest::Client,
    proxy_client: reqwest::Client,
) {
    let Some((_lock, lock_path)) = try_acquire_lock(nsfw) else {
        return;
    };

    if !downloader::check_network(&direct_client).await {
        let _ = std::fs::remove_file(&lock_path);
        return;
    }

    let stock = cache.stock_count(nsfw);

    if stock < config.min_trigger_limit as usize {
        // Resolve the Result before any await to drop the non-Send Box<dyn Error>
        let urls = api_client.random_images(nsfw, config.download_batch_size).await
            .unwrap_or_default();

        for url in urls {
            let filename = cache.generate_filename(nsfw);
            let save_path = cache.stock_dir(nsfw).join(filename);
            let _ = downloader::download_with_retry(
                &url,
                &save_path,
                &proxy_client,
                &direct_client,
                3,
            )
            .await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    let _ = cache.cleanup_excess_stock(nsfw, config.max_cache_limit);

    let _ = std::fs::remove_file(&lock_path);
}

// ─── Daemon Loop ─────────────────────────────────────────────────────

/// Background daemon loop: periodically checks heartbeat and replenishes
/// stock for both SFW and NSFW. Exits when no heartbeat is received for
/// longer than `duration` seconds.
async fn run_daemon(
    duration: u64,
) {
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::path::PathBuf;

    // Set up cache
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("fwaifu");
    let cache = Arc::new(cache::CacheManager::new(cache_dir));
    if let Err(e) = cache.init() {
        eprintln!("{}", i18n::tf("error.daemon_cache_failed", &[&e.to_string()]));
        daemon::cleanup_daemon();
        std::process::exit(1);
    }

    // Set up config (just like main, minus login/status/clean/save which daemon doesn't need)
    let cli = Cli::parse();
    let config = config::Config::load(&cli);

    // Set up HTTP clients
    let direct_client = reqwest::Client::builder()
        .user_agent(concat!("fwaifu/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Daemon: failed to create HTTP client");

    let proxy_client = match config.proxy.as_ref() {
        Some(proxy_url) => {
            let proxy = reqwest::Proxy::all(proxy_url).expect("Daemon: invalid proxy URL");
            reqwest::Client::builder()
                .user_agent(concat!("fwaifu/", env!("CARGO_PKG_VERSION")))
                .proxy(proxy)
                .build()
                .expect("Daemon: failed to create proxy HTTP client")
        }
        None => direct_client.clone(),
    };

    let token_path: Option<PathBuf> = Some(
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("fwaifu")
            .join("token"),
    );

    let api_client = Arc::new(
        api::NekosMoeClient::new(config.proxy.as_deref(), token_path)
            .expect("Daemon: failed to create API client"),
    );

    // Main daemon loop
    loop {
        let heartbeat = daemon::read_heartbeat();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Exit if no heartbeat within the configured duration
        if now.saturating_sub(heartbeat) > duration {
            daemon::cleanup_daemon();
            std::process::exit(0);
        }

        // Check network before trying to download
        if !downloader::check_network(&direct_client).await {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Replenish both SFW and NSFW stock
        for nsfw in [false, true] {
            let stock = cache.stock_count(nsfw);
            if stock < config.min_trigger_limit as usize {
                tokio::spawn(background_fill(
                    nsfw,
                    config.clone(),
                    Arc::clone(&cache),
                    Arc::clone(&api_client),
                    direct_client.clone(),
                    proxy_client.clone(),
                ));
            }
        }

        // Sleep before next check
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// ─── One Cycle ──────────────────────────────────────────────────────

/// Execute one full cycle: pick an image, display it via fastfetch,
/// cleanup, and optionally fall back to the default logo on errors.
async fn run_one_cycle(
    nsfw: bool,
    config: config::Config,
    cache: Arc<cache::CacheManager>,
    api_client: Arc<api::NekosMoeClient>,
    direct_client: reqwest::Client,
    proxy_client: reqwest::Client,
) {
    let stock = cache.stock_count(nsfw);

    let selected: Option<PathBuf>;

    if stock > 0 {
        // ── Stock available: pick one and replenish in background ──
        selected = cache.select_random(nsfw);

        // Background replenisher handles stock refill — no spawn here.
    } else {
        // ── Stock empty: download one image on demand ──
        println!("{}", i18n::t("msg.stock_empty"));

        if !downloader::check_network(&direct_client).await {
            println!("{}", i18n::t("msg.network_error"));
            let _ = runner::run_fastfetch_default(&config.fastfetch_args);
            return;
        }

        // Resolve the Result first to drop the non-Send Box<dyn Error>
        let urls = match api_client.random_images(nsfw, 1).await {
            Ok(urls) => urls,
            Err(_) => {
                fallback_default(&config.fastfetch_args);
                return;
            }
        };

        let Some(url) = urls.into_iter().next() else {
            fallback_default(&config.fastfetch_args);
            return;
        };

        let filename = cache.generate_filename(nsfw);
        let save_path = cache.stock_dir(nsfw).join(&filename);

        match downloader::download_with_retry(
            &url,
            &save_path,
            &proxy_client,
            &direct_client,
            3,
        )
        .await
        {
            Ok(_) => {
                selected = cache.select_random(nsfw);

                // Background replenisher handles stock refill — no spawn here.
            }
            Err(_) => {
                fallback_default(&config.fastfetch_args);
                return;
            }
        }
    }

    // ── Display the selected image (or fallback) ──
    if let Some(selected_path) = selected {
        if config.crop {
            if image::is_imagemagick_available() {
                let _ = image::crop_image(&selected_path, config.crop_width, config.crop_height);
            } else {
                eprintln!("{}", i18n::t("error.imagemagick_not_found"));
            }
        }

        // Display with chafa or fastfetch
        if config.term {
            match runner::capture_chafa(&selected_path, config.crop_width, config.crop_height, config.term_width) {
                Ok(left_lines) => {
                    match runner::capture_fastfetch(&config.fastfetch_args) {
                        Ok(right_lines) => {
                            let merged = runner::merge_side_by_side(&left_lines, &right_lines, 2);
                            print!("{merged}");
                        }
                        Err(e) => {
                            eprintln!("{}", i18n::tf("error.fastfetch_error", &[&e.to_string()]));
                            let _ = runner::run_fastfetch_default(&config.fastfetch_args);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", i18n::tf("error.chafa_error", &[&e.to_string()]));
                }
            }
        } else {
            match runner::run_fastfetch(&selected_path, Some(config.logo_width), &config.fastfetch_args)
            {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("{}", i18n::tf("error.fastfetch_error", &[&e.to_string()]));
                    let _ = runner::run_fastfetch_default(&config.fastfetch_args);
                }
            }
        }

        // Move the used image to the used directory
        let _ = cache.move_to_used(&selected_path, nsfw);

        // Touch the used file so find_last_displayed() can find it
        if let Some(filename) = selected_path.file_name() {
            let used_path = cache.used_dir(nsfw).join(filename);
            let _ = std::fs::OpenOptions::new()
                .write(true)
                .open(&used_path)
                .and_then(|f| f.set_modified(std::time::SystemTime::now()));
        }

        // Cleanup old used images
        let _ = cache.cleanup_used(nsfw, config.max_used_limit);

        // Clean fastfetch thumbnail cache if configured
        if config.clean_cache
            && let Some(ff_cache) = dirs::cache_dir()
        {
            let ff_img_dir = ff_cache.join("fastfetch").join("images");
            if ff_img_dir.exists() {
                let _ = std::fs::remove_dir_all(&ff_img_dir);
            }
        }
    } else {
        fallback_default(&config.fastfetch_args);
    }
}

/// Common fallback: print message and show the default fastfetch logo.
fn fallback_default(fastfetch_args: &[String]) {
    println!("{}", i18n::t("msg.image_failed"));
    let _ = runner::run_fastfetch_default(fastfetch_args);
}

// ─── Update Checker ──────────────────────────────────────────────────

/// Fetch remote Cargo.toml, parse version, compare with current, and prompt to install if newer.
/// When `force` is true, skip version checking and proceed directly to installation.
async fn check_and_update(force: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    let remote_url = "https://raw.githubusercontent.com/cublueer/fwaifu/main/Cargo.toml";

    // Skip version check when force is enabled
    if force {
        println!("Forcing reinstall of version {}...", current_version);
    } else {
        // Fetch remote Cargo.toml
        let client = reqwest::Client::new();
        let resp = match client.get(remote_url).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
                return;
            }
        };

        let body = match resp.text().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to read remote version: {}", e);
                return;
            }
        };

        // Parse version from Cargo.toml
        let remote_version = body
            .lines()
            .find(|line| line.trim().starts_with("version"))
            .and_then(|line| line.split('"').nth(1))
            .unwrap_or("0.0.0");

        println!("Current version: {}", current_version);
        println!("Latest version:  {}", remote_version);

        if remote_version <= current_version {
            println!("Already up to date.");
            return;
        }
    }

    if !command_exists("git") {
        println!("git is not installed. Install it first, or update manually:");
        println!("  curl -fsSL https://raw.githubusercontent.com/cublueer/fwaifu/main/install.sh | bash");
        return;
    }

    {
        use std::io::{self, Write};
        print!("Install update? [y/N] ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || !input.trim().eq_ignore_ascii_case("y") {
            println!("Update cancelled.");
            return;
        }
    }

    let temp_dir = std::env::temp_dir().join(format!("fwaifu_update_{}", std::process::id()));
    println!("Cloning repository...");
    let clone_status = std::process::Command::new("git")
        .args(["clone", "--depth", "1", "https://github.com/cublueer/fwaifu.git"])
        .arg(&temp_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match clone_status {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!("Failed to clone repository. Update manually:");
            eprintln!("  curl -fsSL https://raw.githubusercontent.com/cublueer/fwaifu/main/install.sh | bash");
            let _ = std::fs::remove_dir_all(&temp_dir);
            return;
        }
    }

    println!("Running install.sh...");
    let install_status = std::process::Command::new("bash")
        .arg(temp_dir.join("install.sh"))
        .current_dir(&temp_dir)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    let _ = std::fs::remove_dir_all(&temp_dir);

    if install_status.map_or(false, |s| s.success()) {
        println!("Update complete!");
    } else {
        eprintln!("Install failed. You can try manually:");
        eprintln!("  curl -fsSL https://raw.githubusercontent.com/cublueer/fwaifu/main/install.sh | bash");
    }
}

// ─── Main ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Check for --help or -h before clap parsing to show custom bilingual help
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    // 1. Parse CLI args
    let cli = cli::Cli::parse();

    // Handle --update
    if cli.update {
        check_and_update(cli.force).await;
        return;
    }

    // Handle --completion
    if let Some(shell) = cli.completion {
        use cli::Shell;
        let mut cmd = cli::Cli::command();
        let bin_name = "fwaifu";
        let mut stdout = std::io::stdout();
        match shell {
            Shell::Bash => generate(ClapShell::Bash, &mut cmd, bin_name, &mut stdout),
            Shell::Zsh => generate(ClapShell::Zsh, &mut cmd, bin_name, &mut stdout),
            Shell::Fish => generate(ClapShell::Fish, &mut cmd, bin_name, &mut stdout),
        }
        return;
    }

    // Read daemon duration from environment variable (default 30 seconds)
    let daemon_duration: u64 = std::env::var("FWAIFU_DAEMON_DURATION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    // Internal daemon mode: run as background replenisher
    if cli.daemon {
        daemon::write_daemon_pid();
        run_daemon(daemon_duration).await;
        return;
    }

    // 2. Load Config (file → env → CLI)
    let config = config::Config::load(&cli);

    // Validate proxy URL early so users get a clear error before any network call
    if let Some(ref proxy_url) = config.proxy
        && (!proxy_url.starts_with("http://") && !proxy_url.starts_with("https://"))
    {
        eprintln!("{}", i18n::tf("error.proxy_invalid", &[proxy_url]));
        eprintln!("{}", i18n::t("error.proxy_requires_http"));
        std::process::exit(1);
    }

    // Handle --clean
    if let Some(ref target) = cli.clean {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("fwaifu");
        let cache = cache::CacheManager::new(cache_dir);

        if target == "__DEFAULT__" {
            // No argument: clean both SFW and NSFW
            let _ = cache.clean_all(false);
            let _ = cache.clean_all(true);
            println!("{}", i18n::t("msg.clean_all"));
            return;
        }

        let is_nsfw = match target.to_lowercase().as_str() {
            "nsfw" => true,
            "sfw" => false,
            _ => {
                eprintln!("{}", i18n::tf("error.clean_invalid", &[target]));
                std::process::exit(1);
            }
        };
        if let Err(e) = cache.clean_all(is_nsfw) {
            eprintln!("{}", i18n::tf("error.cache_init_failed", &[&e.to_string()]));
            std::process::exit(1);
        }
        println!("{}", if is_nsfw { i18n::t("msg.clean_nsfw") } else { i18n::t("msg.clean_sfw") });
        return;
    }

    // Handle --save / -s
    if let Some(ref save_arg) = cli.save {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("fwaifu");
        let cache = cache::CacheManager::new(cache_dir);

        let (src, is_nsfw) = match cache.find_last_displayed() {
            Some(result) => result,
            None => {
                println!("{}", i18n::t("msg.no_image_to_save"));
                return;
            }
        };

        let save_dir = if save_arg == "__DEFAULT__" {
            // Use type-specific config, with shared fallback
            let default_dir = || {
                dirs::picture_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("fwaifu")
            };
            let cfg_path = if is_nsfw {
                config.save_path_nsfw.as_deref()
            } else {
                config.save_path_sfw.as_deref()
            };
            cfg_path.map(PathBuf::from).unwrap_or_else(default_dir)
        } else {
            PathBuf::from(save_arg)
        };

        let filename = src.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown.jpg"));
        let dest = save_dir.join(filename);

        match std::fs::create_dir_all(&save_dir)
            .and_then(|_| std::fs::copy(&src, &dest))
        {
            Ok(_) => {
                println!("{}", i18n::tf("msg.image_saved", &[&dest.display().to_string()]));
            }
            Err(e) => {
                eprintln!("{}", i18n::tf("error.save_failed", &[&e.to_string()]));
            }
        }
        return;
    }

    // Check that fastfetch is installed before doing anything
    if !command_exists("fastfetch") {
        eprintln!("{}", i18n::t("error.fastfetch_not_found"));
        std::process::exit(1);
    }

    // Check that chafa is installed when --term is used
    if cli.term && !runner::is_chafa_available() {
        eprintln!("{}", i18n::t("error.chafa_not_found"));
        std::process::exit(1);
    }

    // 3. Handle --status (before --login so it's independent)
    if cli.status {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("fwaifu");
        let token_path = cache_dir.join("token");

        let logged_in = std::fs::read_to_string(&token_path)
            .ok()
            .map(|s| s.trim().to_string())
            .is_some_and(|s| !s.is_empty());

        if logged_in {
            println!("{}", i18n::t("msg.logged_in"));
        } else {
            println!("{}", i18n::t("msg.not_logged_in"));
        }
        return;
    }

    // 4. Handle --login
    if cli.login {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("fwaifu");
        let token_path = cache_dir.join("token");

        let auth_manager = auth::AuthManager::new(token_path.clone());
        let login_client = api::NekosMoeClient::new(config.proxy.as_deref(), Some(token_path))
            .expect("Failed to create API client for login");

        auth_manager
            .interactive_login(&login_client)
            .await
            .unwrap_or_else(|e| eprintln!("{}", i18n::tf("error.login_error", &[&e.to_string()])));
        return;
    }

    // 5. Handle --logout
    if cli.logout {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("fwaifu");
        let token_path = cache_dir.join("token");

        let auth_manager = auth::AuthManager::new(token_path);
        match auth_manager.clear_token() {
            Ok(()) => {
                println!("{}", i18n::t("msg.logged_out"));
            }
            Err(e) => {
                eprintln!("{}", i18n::tf("msg.token_clear_failed", &[&e.to_string()]));
            }
        }
        return;
    }

    // 6. Determine mode
    let nsfw = config.nsfw;
    let watch = config.watch;

    // 7. Set up clients
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("fwaifu");
    let token_path: Option<PathBuf> = Some(cache_dir.join("token"));

    let cache = Arc::new(cache::CacheManager::new(cache_dir));

    let direct_client = reqwest::Client::builder()
        .user_agent(concat!("fwaifu/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to create direct HTTP client");

    let proxy_client = match config.proxy.as_ref() {
        Some(proxy_url) => {
            let proxy = reqwest::Proxy::all(proxy_url).expect("Invalid proxy URL");
            reqwest::Client::builder()
                .user_agent(concat!("fwaifu/", env!("CARGO_PKG_VERSION")))
                .proxy(proxy)
                .build()
                .expect("Failed to create proxy HTTP client")
        }
        None => direct_client.clone(),
    };

    let api_client = Arc::new(
        api::NekosMoeClient::new(config.proxy.as_deref(), token_path.clone())
            .expect("Failed to create API client"),
    );

    // 8. Initialize cache directories
    if let Err(e) = cache.init() {
        eprintln!("{}", i18n::tf("error.cache_init_failed", &[&e.to_string()]));
    }

    // 9. Build the run_one_cycle closure (clones captured state on each call for watch mode)
    let run_once = {
        let cfg = config.clone();
        let c = Arc::clone(&cache);
        let api = Arc::clone(&api_client);
        let dc = direct_client.clone();
        let pc = proxy_client.clone();

        move || {
            let cfg = cfg.clone();
            let c = Arc::clone(&c);
            let api = Arc::clone(&api);
            let dc = dc.clone();
            let pc = pc.clone();

            async move {
                run_one_cycle(nsfw, cfg, c, api, dc, pc).await;
            }
        }
    };

    // 10 / 11. Watch loop or single run
    if watch {
        watch::watch_loop(config.watch_interval, run_once).await;
    } else {
        // Ensure daemon is running for background replenishment
        daemon::ensure_daemon(daemon_duration);
        run_once().await;
        // Exit immediately — daemon handles replenishment in the background
    }
}
