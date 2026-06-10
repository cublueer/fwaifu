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

/// Check if chafa is available on the system.
pub fn is_chafa_available() -> bool {
    Command::new("chafa")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Detect terminal width in columns. Falls back to 80.
fn detect_term_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

/// Capture chafa output as a vector of lines.
/// If `term_width_override` is provided, use it as the image width.
/// Otherwise, auto-detect terminal width and use 1/3 of it for the image.
/// Height is calculated from the crop dimensions' aspect ratio.
pub fn capture_chafa(
    image_path: &PathBuf,
    crop_width: u32,
    crop_height: u32,
    term_width_override: Option<u32>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let img_width = term_width_override.unwrap_or_else(|| {
        detect_term_width().saturating_div(3).max(20) as u32
    });
    let img_height = if crop_width > 0 {
        (img_width as u64 * crop_height as u64 / crop_width as u64 / 2 * 2) as u32
    } else {
        img_width * 2
    };
    let size_arg = format!("{img_width}x{img_height}");

    let output = Command::new("chafa")
        .arg("-s")
        .arg(&size_arg)
        .arg("--format")
        .arg("symbols")
        .arg("--relative")
        .arg("off")
        .arg(image_path)
        .output()
        .map_err(|_| "chafa is not installed or not found on PATH.".to_string())?;

    if !output.status.success() {
        return Err("chafa exited with non-zero status".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();

    Ok(lines)
}

/// Capture fastfetch output (with -l none to disable logo) as a vector of lines.
pub fn capture_fastfetch(
    extra_args: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("fastfetch")
        .arg("-l")
        .arg("none")
        .args(extra_args)
        .output()
        .map_err(|_| "fastfetch is not installed or not found on PATH.".to_string())?;

    if !output.status.success() {
        return Err("fastfetch exited with non-zero status".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();

    Ok(lines)
}

/// Merge two line vectors side by side, with a gap between them.
/// Left lines get an ANSI reset appended to prevent color bleed.
/// When left lines run out, pads with spaces matching the visible width
/// of the widest left line so the right side stays aligned.
pub fn merge_side_by_side(
    left: &[String],
    right: &[String],
    gap: usize,
) -> String {
    let max_lines = left.len().max(right.len());
    let gap_str = " ".repeat(gap);

    let left_width = left.iter()
        .map(|s| visible_width(s))
        .max()
        .unwrap_or(0);

    let blank_left = " ".repeat(left_width);
    let mut result = String::new();

    for i in 0..max_lines {
        let l = left.get(i).map(|s| s.as_str()).unwrap_or("");

        if !l.is_empty() {
            let lw = visible_width(l);
            result.push_str(l);
            result.push_str("\x1b[0m");
            if lw < left_width {
                result.push_str(&" ".repeat(left_width - lw));
            }
            result.push_str(&gap_str);
        } else {
            result.push_str(&blank_left);
            result.push_str(&gap_str);
        }

        if let Some(r) = right.get(i) {
            result.push_str(r);
        }
        result.push('\n');
    }

    result
}

/// Count visible characters in a string, stripping ANSI escape sequences.
fn visible_width(s: &str) -> usize {
    let mut count = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }
        count += 1;
    }
    count
}
