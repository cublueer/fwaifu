use std::io::Write;

use crate::api::NekosMoeClient;

fn is_chinese() -> bool {
    std::env::var("LANG")
        .unwrap_or_default()
        .to_lowercase()
        .starts_with("zh")
}

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
        std::fs::write(&self.token_path, token)
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
            if is_chinese() {
                print!("请输入 Username: ");
            } else {
                print!("Username: ");
            }
            stdout.flush()?;
            stdin.read_line(&mut buf)?;
            buf.trim().to_string()
        };

        let password = {
            let prompt = if is_chinese() {
                "请输入 Password: "
            } else {
                "Password: "
            };
            rpassword::prompt_password(prompt)?
                .trim()
                .to_string()
        };

        match client.login(&username, &password).await {
            Ok(token) => {
                self.save_token(&token)?;
                if is_chinese() {
                    println!("\n✅ 登录成功！");
                } else {
                    println!("\n✅ Login successful!");
                }
                Ok(())
            }
            Err(e) => {
                if is_chinese() {
                    println!("\n❌ 登录失败：{e}");
                } else {
                    println!("\n❌ Login failed: {e}");
                }
                Err(e)
            }
        }
    }
}

