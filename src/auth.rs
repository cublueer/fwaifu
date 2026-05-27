use std::io::{self, Write};

use std::os::unix::fs::PermissionsExt;

use crate::api::NekosMoeClient;

pub struct AuthManager {
    token_path: std::path::PathBuf,
}

impl AuthManager {
    pub fn new(token_path: std::path::PathBuf) -> Self {
        AuthManager { token_path }
    }

    #[allow(dead_code)]
    pub fn load_token(&self) -> Option<String> {
        let content = std::fs::read_to_string(&self.token_path).ok()?;
        let token = content.trim();
        if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        }
    }

    pub fn save_token(&self, token: &str) -> std::io::Result<()> {
        if let Some(parent) = self.token_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.token_path, token)?;
        // Restrict token file to owner-only read/write (0600)
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&self.token_path, perms)?;
        Ok(())
    }

    pub fn clear_token(&self) -> std::io::Result<()> {
        if self.token_path.exists() {
            std::fs::remove_file(&self.token_path)?;
        }
        Ok(())
    }

    pub async fn interactive_login(
        &self,
        client: &NekosMoeClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let username = {
            let mut buf = String::new();
            print!("{}", crate::i18n::t("prompt.username"));
            stdout.flush()?;
            stdin.read_line(&mut buf)?;
            buf.trim().to_string()
        };

        let password = read_password_with_stars(crate::i18n::t("prompt.password"))?;

        match client.login(&username, &password).await {
            Ok(token) => {
                self.save_token(&token)?;
                println!("\n{}", crate::i18n::t("msg.login_success"));
                Ok(())
            }
            Err(e) => {
                println!("\n{}", crate::i18n::tf("msg.login_failed", &[&e.to_string()]));
                Err(e)
            }
        }
    }
}

/// Read a password from the terminal, displaying `●` for each character.
///
/// Enables raw terminal mode (no echo, no line buffering) to capture
/// individual keystrokes. Restores the original terminal settings on exit
/// (even on panic).
fn read_password_with_stars(prompt: &str) -> io::Result<String> {
    let mut stdout = io::stdout();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;

    // ── Save original terminal settings ──
    let stdin_fd = libc::STDIN_FILENO;
    let termios_orig = unsafe {
        let mut t = std::mem::zeroed::<libc::termios>();
        if libc::tcgetattr(stdin_fd, &mut t) != 0 {
            return Err(io::Error::last_os_error());
        }
        t
    };

    // ── Switch to raw mode: disable canonical (line-buffered) and echo ──
    let mut termios_raw = termios_orig;
    termios_raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    termios_raw.c_cc[libc::VMIN] = 1;
    termios_raw.c_cc[libc::VTIME] = 0;
    if unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios_raw) } != 0 {
        return Err(io::Error::last_os_error());
    }

    // ── Restore original terminal on scope exit ──
    struct RestoreTerminal {
        termios: libc::termios,
        fd: libc::c_int,
    }
    impl Drop for RestoreTerminal {
        fn drop(&mut self) {
            unsafe {
                libc::tcsetattr(self.fd, libc::TCSANOW, &self.termios);
            }
        }
    }
    let _restore = RestoreTerminal {
        termios: termios_orig,
        fd: stdin_fd,
    };

    // ── Read characters one at a time ──
    let mut password = String::new();
    let mut buf = [0u8; 1];

    loop {
        let n = unsafe { libc::read(stdin_fd, buf.as_mut_ptr() as *mut libc::c_void, 1) };
        if n <= 0 {
            break;
        }

        match buf[0] {
            b'\n' | b'\r' => break,
            0x7f | b'\x08' => {
                if !password.is_empty() {
                    password.pop();
                    write!(stdout, "\x08 \x08")?;
                    stdout.flush()?;
                }
            }
            0x03 => {
                write!(stdout, "^C\n")?;
                stdout.flush()?;
                std::process::exit(1);
            }
            b if b.is_ascii_graphic() || b == b' ' => {
                password.push(b as char);
                stdout.write_all("●".as_bytes())?;
                stdout.flush()?;
            }
            _ => {}
        }
    }

    writeln!(stdout)?;
    Ok(password)
}

