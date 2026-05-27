use serde::{Deserialize, Serialize};

use crate::cli::Cli;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub nsfw: bool,
    pub watch: bool,
    pub watch_interval: u64,
    pub proxy: Option<String>,
    pub crop: bool,
    pub crop_width: u32,
    pub crop_height: u32,
    pub logo_width: u32,
    pub download_batch_size: u32,
    pub max_cache_limit: u32,
    pub min_trigger_limit: u32,
    pub max_used_limit: u32,
    pub clean_cache: bool,
    pub save_path_sfw: Option<String>,
    pub save_path_nsfw: Option<String>,
    pub fastfetch_args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            nsfw: false,
            watch: false,
            watch_interval: 5,
            proxy: None,
            crop: true,
            crop_width: 600,
            crop_height: 800,
            logo_width: 40,
            download_batch_size: 10,
            max_cache_limit: 100,
            min_trigger_limit: 60,
            max_used_limit: 50,
            clean_cache: true,
            save_path_sfw: None,
            save_path_nsfw: None,
            fastfetch_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FileConfig {
    proxy: Option<String>,
    crop: Option<bool>,
    crop_width: Option<u32>,
    crop_height: Option<u32>,
    logo_width: Option<u32>,
    watch_interval: Option<u64>,
    #[serde(default)]
    download: FileDownloadConfig,
    #[serde(default)]
    cache: FileCacheConfig,
    save_path_sfw: Option<String>,
    save_path_nsfw: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct FileDownloadConfig {
    batch_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct FileCacheConfig {
    max_limit: Option<u32>,
    min_trigger: Option<u32>,
    max_used: Option<u32>,
    clean_cache: Option<bool>,
}

impl Config {
    pub fn load(cli: &Cli) -> Self {
        let mut config = Config::default();

        // Step 2: Merge from config file (~/.config/fwaifu/config.toml)
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("fwaifu").join("config.toml");
            if config_path.exists()
                && let Ok(content) = std::fs::read_to_string(&config_path)
                && let Ok(file_config) = toml::from_str::<FileConfig>(&content)
            {
                merge_file_config(&mut config, &file_config);
            }
        }

        // Step 3: Env var override (FWAIFU_PROXY)
        if let Ok(proxy) = std::env::var("FWAIFU_PROXY") {
            config.proxy = Some(proxy);
        }

        // Step 4: CLI overrides (highest priority)
        merge_cli_overrides(&mut config, cli);

        config
    }
}

fn merge_file_config(config: &mut Config, file: &FileConfig) {
    if let Some(ref proxy) = file.proxy {
        config.proxy = Some(proxy.clone());
    }
    if let Some(crop) = file.crop {
        config.crop = crop;
    }
    if let Some(crop_width) = file.crop_width {
        config.crop_width = crop_width;
    }
    if let Some(crop_height) = file.crop_height {
        config.crop_height = crop_height;
    }
    if let Some(logo_width) = file.logo_width {
        config.logo_width = logo_width;
    }
    if let Some(watch_interval) = file.watch_interval {
        config.watch_interval = watch_interval;
    }
    if let Some(batch_size) = file.download.batch_size {
        config.download_batch_size = batch_size;
    }
    if let Some(max_limit) = file.cache.max_limit {
        config.max_cache_limit = max_limit;
    }
    if let Some(min_trigger) = file.cache.min_trigger {
        config.min_trigger_limit = min_trigger;
    }
    if let Some(max_used) = file.cache.max_used {
        config.max_used_limit = max_used;
    }
    if let Some(clean_cache) = file.cache.clean_cache {
        config.clean_cache = clean_cache;
    }
    if let Some(ref path) = file.save_path_sfw {
        config.save_path_sfw = Some(path.clone());
    }
    if let Some(ref path) = file.save_path_nsfw {
        config.save_path_nsfw = Some(path.clone());
    }
}

fn merge_cli_overrides(config: &mut Config, cli: &Cli) {
    if cli.nsfw {
        config.nsfw = true;
    }
    if cli.watch {
        config.watch = true;
    }
    if let Some(watch_interval) = cli.watch_interval {
        config.watch_interval = watch_interval;
    }
    if let Some(ref proxy) = cli.proxy {
        config.proxy = Some(proxy.clone());
    }
    if cli.no_crop {
        config.crop = false;
    }
    if let Some(crop_width) = cli.crop_width {
        config.crop_width = crop_width;
    }
    if let Some(crop_height) = cli.crop_height {
        config.crop_height = crop_height;
    }
    if let Some(logo_width) = cli.logo_width {
        config.logo_width = logo_width;
    }
    if !cli.fastfetch_args.is_empty() {
        config.fastfetch_args = cli.fastfetch_args.clone();
    }
}
