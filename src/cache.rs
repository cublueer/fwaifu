use rand::Rng;
use std::path::PathBuf;

/// Manages the local image cache under a base directory (~/.cache/fwaifu).
///
/// Organizes images into stock and used sub-directories, separated by
/// SFW and NSFW content.
pub struct CacheManager {
    base_dir: PathBuf,
}

impl CacheManager {
    /// Create a new CacheManager with the given base directory.
    ///
    /// The base directory is typically `~/.cache/fwaifu`.
    pub fn new(base_dir: PathBuf) -> Self {
        CacheManager { base_dir }
    }

    /// Get the stock directory path for a given mode.
    ///
    /// Returns `base_dir/sfw` or `base_dir/nsfw`.
    pub fn stock_dir(&self, nsfw: bool) -> PathBuf {
        let dir_name = if nsfw { "nsfw" } else { "sfw" };
        self.base_dir.join(dir_name)
    }

    /// Get the used directory path for a given mode.
    ///
    /// Returns `base_dir/used/sfw` or `base_dir/used/nsfw`.
    pub fn used_dir(&self, nsfw: bool) -> PathBuf {
        let dir_name = if nsfw { "nsfw" } else { "sfw" };
        self.base_dir.join("used").join(dir_name)
    }

    /// Initialize all cache directories.
    ///
    /// Creates stock and used directories for both SFW and NSFW content.
    pub fn init(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.stock_dir(false))?;
        std::fs::create_dir_all(self.stock_dir(true))?;
        std::fs::create_dir_all(self.used_dir(false))?;
        std::fs::create_dir_all(self.used_dir(true))?;
        Ok(())
    }

    /// Count JPG files (by `.jpg` extension, case-insensitive) in the stock directory.
    pub fn stock_count(&self, nsfw: bool) -> usize {
        count_jpg_files(&self.stock_dir(nsfw))
    }

    /// Select a random JPG file from the stock directory.
    ///
    /// Returns `None` if the directory is empty or cannot be read.
    pub fn select_random(&self, nsfw: bool) -> Option<PathBuf> {
        let files = list_jpg_files(&self.stock_dir(nsfw));
        if files.is_empty() {
            return None;
        }

        let idx = rand::thread_rng().gen_range(0..files.len());
        Some(files[idx].clone())
    }

    /// Move an image file from its current path to the used directory.
    ///
    /// The file keeps its original filename. The used directory is created
    /// if it does not exist.
    pub fn move_to_used(&self, file_path: &PathBuf, nsfw: bool) -> std::io::Result<()> {
        let filename = file_path
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "File path has no filename"))?;

        let dest = self.used_dir(nsfw).join(filename);

        std::fs::create_dir_all(self.used_dir(nsfw))?;
        std::fs::rename(file_path, &dest)
    }

    /// Cleanup the used directory: if it has more than `max_used` files,
    /// delete the oldest ones (by modification time) until count ≤ max_used.
    pub fn cleanup_used(&self, nsfw: bool, max_used: u32) -> std::io::Result<()> {
        cleanup_dir(&self.used_dir(nsfw), max_used as usize)
    }

    /// Cleanup the stock directory: if it has more than `max_limit` files,
    /// delete the oldest ones until count ≤ max_limit.
    pub fn cleanup_excess_stock(&self, nsfw: bool, max_limit: u32) -> std::io::Result<()> {
        cleanup_dir(&self.stock_dir(nsfw), max_limit as usize)
    }

    /// Generate a unique filename.
    ///
    /// SFW format: `waifu_{timestamp_nanos}_{random}.jpg`
    /// NSFW format: `waifu_nsfw_{timestamp_nanos}_{random}.jpg`
    pub fn generate_filename(&self, nsfw: bool) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let random: u32 = rand::random();

        let prefix = if nsfw { "waifu_nsfw" } else { "waifu" };
        format!("{prefix}_{nanos}_{random:x}.jpg")
    }
}

/// List all `.jpg` files (case-insensitive extension check) in a directory.
fn list_jpg_files(dir: &PathBuf) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let ext = path.extension()?.to_str()?;
            if ext.eq_ignore_ascii_case("jpg") {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

/// Count JPG files in a directory.
fn count_jpg_files(dir: &PathBuf) -> usize {
    list_jpg_files(dir).len()
}

/// Remove the oldest files from a directory until file count ≤ max_count.
///
/// Files are sorted by modification time (oldest first). Excess files
/// beyond `max_count` are deleted.
fn cleanup_dir(dir: &PathBuf, max_count: usize) -> std::io::Result<()> {
    let mut files = list_jpg_files(dir);

    if files.len() <= max_count {
        return Ok(());
    }

    // Sort by modification time, oldest first
    files.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let to_remove = files.len() - max_count;
    for path in files.iter().take(to_remove) {
        if let Err(e) = std::fs::remove_file(path) {
            eprintln!("Warning: failed to remove {}: {e}", path.display());
        }
    }

    Ok(())
}
