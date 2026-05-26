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

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

// ─── Helpers ─────────────────────────────────────────────────────────

fn is_chinese() -> bool {
    std::env::var("LANG")
        .unwrap_or_default()
        .to_lowercase()
        .starts_with("zh")
}

fn print_help() {
    if is_chinese() {
        println!("========================================================");
        println!("    fwaifu - 随机二次元美少女生成器暨 Fastfetch 终端看板娘");
        println!("========================================================");
        println!();
        println!("用法模式：");
        println!("  fwaifu              标准模式。随机生成一张 SFW 美少女图片同时显示系统信息。");
        println!("  fwaifu -n / --nsfw  NSFW 模式。显示 NSFW 图片（需要先 --login 登录）。");
        println!("  fwaifu -w / --watch 持续轮播模式。每 N 秒刷新，适合挂在副屏当作动态看板娘。");
        println!("  fwaifu -w -n         持续 NSFW 轮播模式。");
        println!();
        println!("选项：");
        println!("  -h, --help                  显示本帮助信息");
        println!("  -n, --nsfw                  启用 NSFW 模式");
        println!("  -w, --watch                 启用轮播模式");
        println!("  --watch-interval <SECONDS>  轮播间隔秒数 (默认 5)");
        println!("  -p, --proxy <URL>           设置代理地址 (如 http://127.0.0.1:7890)");
        println!("  --no-crop                   关闭图片裁剪");
        println!("  --crop-width <WIDTH>        裁剪目标宽度 (默认 600)");
        println!("  --crop-height <HEIGHT>      裁剪目标高度 (默认 800)");
        println!("  --logo-width <WIDTH>        Fastfetch 图片展示宽度 (默认 40)");
        println!("  --login                     交互式登录 Nekos.moe，保存 Token");
        println!("  --logout                    登出并清除本地 Token 文件");
        println!("  --status                    显示登录状态");
        println!("  --version                   显示版本信息");
        println!();
        println!("配置：");
        println!("  配置文件:   ~/.config/fwaifu/config.toml");
        println!("  环境变量:   FWAIFU_PROXY");
        println!("  优先级:     CLI 参数 > 环境变量 > 配置文件");
        println!();
        println!("图片源： Nekos.moe API (https://nekos.moe)");
        println!("========================================================");
    } else {
        println!("========================================================");
        println!("    fwaifu - Random Anime Girl Generator for Terminal");
        println!("========================================================");
        println!();
        println!("Usage Modes:");
        println!("  fwaifu              Standard mode. Shows a random SFW anime girl with system info.");
        println!("  fwaifu -n / --nsfw  NSFW mode. Shows NSFW images (requires --login first).");
        println!("  fwaifu -w / --watch Continuous watch mode. Refreshes every N seconds, great for a secondary monitor.");
        println!("  fwaifu -w -n         Continuous NSFW watch mode.");
        println!();
        println!("Options:");
        println!("  -h, --help                  Show this help message");
        println!("  -n, --nsfw                  Enable NSFW mode");
        println!("  -w, --watch                 Enable watch mode");
        println!("  --watch-interval <SECONDS>  Watch interval in seconds (default 5)");
        println!("  -p, --proxy <URL>           Set proxy URL (e.g. http://127.0.0.1:7890)");
        println!("  --no-crop                   Disable image cropping");
        println!("  --crop-width <WIDTH>        Crop target width (default 600)");
        println!("  --crop-height <HEIGHT>      Crop target height (default 800)");
        println!("  --logo-width <WIDTH>        Fastfetch logo display width (default 40)");
        println!("  --login                     Interactive login to Nekos.moe (saves token)");
        println!("  --logout                    Logout and clear stored token");
        println!("  --status                    Show login status");
        println!("  --version                   Show version information");
        println!();
        println!("Configuration:");
        println!("  Config file:  ~/.config/fwaifu/config.toml");
        println!("  Environment:  FWAIFU_PROXY");
        println!("  Priority:     CLI args > Environment > Config file");
        println!();
        println!("Image source: Nekos.moe API (https://nekos.moe)");
        println!("========================================================");
    }
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
        eprintln!("Daemon: failed to init cache: {e}");
        daemon::cleanup_daemon();
        std::process::exit(1);
    }

    // Set up config (load from file + env, no CLI overrides in daemon mode)
    let mut config = config::Config::default();
    // Merge from file
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("fwaifu").join("config.toml");
        if config_path.exists()
            && let Ok(content) = std::fs::read_to_string(&config_path)
        {
                // Use a minimal file config struct for daemon mode
                #[derive(serde::Deserialize)]
                struct DaemonFileConfig {
                    proxy: Option<String>,
                    #[serde(default)]
                    download: DaemonDownloadConfig,
                    #[serde(default)]
                    cache: DaemonCacheConfig,
                }
                #[derive(serde::Deserialize, Default)]
                struct DaemonDownloadConfig {
                    batch_size: Option<u32>,
                }
                #[derive(serde::Deserialize, Default)]
                struct DaemonCacheConfig {
                    max_limit: Option<u32>,
                    min_trigger: Option<u32>,
                    max_used: Option<u32>,
                }
                if let Ok(fc) = toml::from_str::<DaemonFileConfig>(&content) {
                    if let Some(proxy) = fc.proxy {
                        config.proxy = Some(proxy);
                    }
                    if let Some(bs) = fc.download.batch_size {
                        config.download_batch_size = bs;
                    }
                    if let Some(ml) = fc.cache.max_limit {
                        config.max_cache_limit = ml;
                    }
                    if let Some(mt) = fc.cache.min_trigger {
                        config.min_trigger_limit = mt;
                    }
                    if let Some(mu) = fc.cache.max_used {
                        config.max_used_limit = mu;
                    }
                }
            }
    }
    // Env var override for proxy
    if let Ok(proxy) = std::env::var("FWAIFU_PROXY") {
        config.proxy = Some(proxy);
    }

    // Set up HTTP clients
    let direct_client = reqwest::Client::builder()
        .user_agent("fwaifu/0.1.0")
        .build()
        .expect("Daemon: failed to create HTTP client");

    let proxy_client = match config.proxy.as_ref() {
        Some(proxy_url) => {
            let proxy = reqwest::Proxy::all(proxy_url).expect("Daemon: invalid proxy URL");
            reqwest::Client::builder()
                .user_agent("fwaifu/0.1.0")
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
        if is_chinese() {
            println!("库存不够啦！正在去搬运新的图片，请稍等哦...");
        } else {
            println!("Not enough stock! Fetching new images, please wait...");
        }

        if !downloader::check_network(&direct_client).await {
            if is_chinese() {
                println!("网络好像不太通畅，无法下载新图片 QAQ");
            } else {
                println!("Network seems unreachable, can't download new images QAQ");
            }
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
        // Crop if configured and ImageMagick is available
        if config.crop && image::is_imagemagick_available() {
            let _ = image::crop_image(&selected_path, config.crop_width, config.crop_height);
        }

        // Run fastfetch with the image
        match runner::run_fastfetch(&selected_path, Some(config.logo_width), &config.fastfetch_args)
        {
            Ok(()) => {}
            Err(e) => {
                eprintln!("fastfetch error: {e}");
                let _ = runner::run_fastfetch_default(&config.fastfetch_args);
            }
        }

        // Move the used image to the used directory
        let _ = cache.move_to_used(&selected_path, nsfw);

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
    if is_chinese() {
        println!("图片获取失败了，这次只能先显示默认的 Logo 啦 QAQ");
    } else {
        println!("Failed to get an image, showing default logo this time QAQ");
    }
    let _ = runner::run_fastfetch_default(fastfetch_args);
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
        eprintln!("Invalid proxy URL: {proxy_url}");
        eprintln!("Proxy must start with http:// or https://");
        std::process::exit(1);
    }

    // Check that fastfetch is installed before doing anything
    if !command_exists("fastfetch") {
        eprintln!("fastfetch is not installed or not found on PATH.");
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
            if is_chinese() {
                println!("✅ 已登录 Nekos.moe");
            } else {
                println!("✅ Logged in to Nekos.moe");
            }
        } else {
            if is_chinese() {
                println!("❌ 未登录 Nekos.moe");
            } else {
                println!("❌ Not logged in to Nekos.moe");
            }
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
            .unwrap_or_else(|e| eprintln!("Login error: {e}"));
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
                if is_chinese() {
                    println!("✅ 已登出，Token 已清除。");
                } else {
                    println!("✅ Logged out. Token cleared.");
                }
            }
            Err(e) => {
                if is_chinese() {
                    eprintln!("⚠️ 清除 Token 失败：{e}");
                } else {
                    eprintln!("⚠️ Failed to clear token: {e}");
                }
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
        .user_agent("fwaifu/0.1.0")
        .build()
        .expect("Failed to create direct HTTP client");

    let proxy_client = match config.proxy.as_ref() {
        Some(proxy_url) => {
            let proxy = reqwest::Proxy::all(proxy_url).expect("Invalid proxy URL");
            reqwest::Client::builder()
                .user_agent("fwaifu/0.1.0")
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
        eprintln!("Warning: failed to initialize cache directories: {e}");
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
