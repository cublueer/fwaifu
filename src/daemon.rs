use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const HEARTBEAT_FILE: &str = "/tmp/fwaifu_heartbeat";
const DAEMON_PID_FILE: &str = "/tmp/fwaifu_daemon.pid";

/// Touch the heartbeat file with the current Unix timestamp (seconds).
pub fn touch_heartbeat() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let _ = std::fs::write(HEARTBEAT_FILE, now.to_string());
}

/// Read the last heartbeat timestamp. Returns 0 if the file doesn't exist or is unreadable.
pub fn read_heartbeat() -> u64 {
    std::fs::read_to_string(HEARTBEAT_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Check whether the daemon process is still alive by reading its PID file
/// and checking `/proc/{pid}`.
pub fn is_daemon_alive() -> bool {
    if let Ok(content) = std::fs::read_to_string(DAEMON_PID_FILE)
        && let Ok(pid) = content.trim().parse::<u32>()
    {
        return std::path::Path::new(&format!("/proc/{pid}")).exists();
    }
    false
}

/// Start the daemon as a detached background process.
/// Uses `std::process::Command` to spawn `fwaifu --daemon` with
/// FWAIFU_DAEMON_DURATION set, redirecting all stdio to /dev/null.
pub fn start_daemon(duration: u64) {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("fwaifu"));
    let _ = std::process::Command::new(exe)
        .arg("--daemon")
        .env("FWAIFU_DAEMON_DURATION", duration.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    // Brief sleep to let the daemon start and write its PID file
    std::thread::sleep(Duration::from_millis(100));
}

/// Write the current process PID to the daemon PID file.
pub fn write_daemon_pid() {
    let pid = std::process::id();
    let _ = std::fs::write(DAEMON_PID_FILE, pid.to_string());
}

/// Clean up daemon files (heartbeat + PID). Called when daemon exits.
pub fn cleanup_daemon() {
    let _ = std::fs::remove_file(HEARTBEAT_FILE);
    let _ = std::fs::remove_file(DAEMON_PID_FILE);
}

/// Ensure the daemon is running: touch heartbeat, start daemon if not alive.
/// Call this from the main `fwaifu` invocation before displaying.
pub fn ensure_daemon(duration: u64) {
    touch_heartbeat();
    if !is_daemon_alive() {
        start_daemon(duration);
    }
}
