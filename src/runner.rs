use std::path::PathBuf;
use std::process::Command;

/// Run fastfetch with the specified image as logo.
/// Passes through any additional fastfetch arguments.
///
/// Returns Ok(()) if fastfetch exits successfully, Err otherwise.
pub fn run_fastfetch(
    image_path: &PathBuf,
    logo_width: Option<u32>,
    extra_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("fastfetch");

    cmd.arg("--logo")
        .arg(image_path)
        .arg("--logo-preserve-aspect-ratio")
        .arg("true");

    if let Some(width) = logo_width {
        cmd.arg("--logo-width").arg(width.to_string());
    }

    cmd.args(extra_args);

    let status = cmd
        .status()
        .map_err(|_| "fastfetch is not installed or not found on PATH.".to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("fastfetch exited with non-zero status: {}", status).into())
    }
}

/// Run fastfetch without any image (fallback/default mode).
pub fn run_fastfetch_default(
    extra_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("fastfetch")
        .args(extra_args)
        .status()
        .map_err(|_| "fastfetch is not installed or not found on PATH.".to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("fastfetch exited with non-zero status: {}", status).into())
    }
}
