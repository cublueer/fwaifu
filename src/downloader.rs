use std::path::PathBuf;

/// Magic bytes for common image formats used for validation.
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47];
const GIF_MAGIC: &[u8] = &[0x47, 0x49, 0x46, 0x38];
const WEBP_MAGIC: &[u8] = &[0x52, 0x49, 0x46, 0x46]; // RIFF

/// Validate that the downloaded bytes represent a valid image format.
///
/// Prefers the system `file` command if available, otherwise falls back
/// to checking magic bytes at the start of the data.
fn validate_image(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if try_file_command(data) {
        return Ok(());
    }

    if data.len() < 4 {
        return Err("Downloaded data is too small to be a valid image".into());
    }

    if data.starts_with(JPEG_MAGIC) || data.starts_with(PNG_MAGIC) || data.starts_with(GIF_MAGIC) || data.starts_with(WEBP_MAGIC) {
        return Ok(());
    }

    Err("Downloaded file is not a valid image".into())
}

/// Attempt to validate using the system `file` command.
///
/// Writes data to a temp file, runs `file --brief --mime-type`, and
/// checks if the output starts with `image/`. Returns true if valid.
fn try_file_command(data: &[u8]) -> bool {
    use std::io::Write;

    let tmp = std::env::temp_dir().join(format!("fwaifu_validate_{}", std::process::id()));
    let Ok(mut f) = std::fs::File::create(&tmp) else {
        return false;
    };
    if f.write_all(data).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    drop(f);

    let output = std::process::Command::new("file")
        .args(["--brief", "--mime-type"])
        .arg(&tmp)
        .output();

    let _ = std::fs::remove_file(&tmp);

    let Ok(output) = output else { return false };

    if !output.status.success() {
        return false;
    }

    let mime = String::from_utf8_lossy(&output.stdout).trim().to_string();

    mime.starts_with("image/")
}

/// Download an image from a URL, save to a file with proper MIME validation.
///
/// Returns the path to the saved file, or an error.
pub async fn download_image(
    client: &reqwest::Client,
    url: &str,
    save_path: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Download returned HTTP {status}").into());
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    validate_image(&bytes)?;

    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
    }

    std::fs::write(save_path, &bytes)
        .map_err(|e| format!("Failed to write file {}: {e}", save_path.display()))?;

    Ok(save_path.clone())
}

/// Check network connectivity by making a HEAD request to a reliable endpoint.
///
/// Returns true if the request succeeds, false otherwise.
pub async fn check_network(client: &reqwest::Client) -> bool {
    let Ok(response) = client
        .head("https://nekos.moe")
        .timeout(std::time::Duration::from_millis(1500))
        .send()
        .await
    else {
        return false;
    };

    response.status().is_success()
}

/// Download with retry logic: try direct first, then with proxy.
///
/// `proxy_client` should be the proxy-configured client, `direct_client` is without proxy.
/// Retries up to `max_retries` times with a 0.5s delay between attempts.
pub async fn download_with_retry(
    url: &str,
    save_path: &PathBuf,
    proxy_client: &reqwest::Client,
    direct_client: &reqwest::Client,
    max_retries: u32,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut last_error: Option<String> = None;

    for _ in 0..max_retries {
        // Try direct client first — fully consume result before next await
        if let Ok(path) = download_image(direct_client, url, save_path).await {
            return Ok(path);
        }

        // Direct failed, try proxy client
        match download_image(proxy_client, url, save_path).await {
            Ok(path) => return Ok(path),
            Err(e) => {
                last_error = Some(e.to_string());
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Err(format!(
        "All {} download attempts failed. Last error: {}",
        max_retries,
        last_error.unwrap_or_else(|| "unknown".to_string())
    )
    .into())
}
