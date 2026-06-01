use std::path::PathBuf;
use std::process::Command;

/// Crop an image to the specified dimensions using ImageMagick.
/// The image is resized to fill the target box, then centered and cropped.
/// Equivalent to: magick input -resize WxH^ -gravity center -extent WxH output
///
/// Returns Ok(()) on success, Err with descriptive message on failure.
pub fn crop_image(
    input_path: &PathBuf,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let resize_arg = format!("{width}x{height}^");
    let extent_arg = format!("{width}x{height}");

    let cmd = find_imagemagick_cmd()?;

    let status = Command::new(&cmd)
        .arg(input_path)
        .arg("-resize")
        .arg(&resize_arg)
        .arg("-gravity")
        .arg("center")
        .arg("-extent")
        .arg(&extent_arg)
        .arg(input_path)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => Ok(()),
        Ok(exit_status) => Err(format!(
            "{} exited with status {} while cropping {:?}",
            cmd, exit_status, input_path
        )
        .into()),
        Err(e) => Err(format!(
            "Failed to run {} for cropping {:?}: {}",
            cmd, input_path, e
        )
        .into()),
    }
}

/// Check if ImageMagick is available on the system.
/// Returns true if either `magick` (IM 7) or `convert` (IM 6) is found.
pub fn is_imagemagick_available() -> bool {
    find_imagemagick_cmd().is_ok()
}

/// Find an available ImageMagick command.
/// Tries `magick` (ImageMagick 7) first, then falls back to `convert` (ImageMagick 6).
fn find_imagemagick_cmd() -> Result<String, Box<dyn std::error::Error>> {
    if command_exists("magick") {
        return Ok("magick".to_string());
    }
    if command_exists("convert") {
        return Ok("convert".to_string());
    }
    Err("ImageMagick is required but not installed. Install it with `apt install imagemagick` or `brew install imagemagick`.".into())
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
