use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;

const BASE_URL: &str = "https://nekos.moe/api/v1";
const USER_AGENT_VALUE: &str = "fwaifu/0.1.0";

pub struct NekosMoeClient {
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct RandomImagesResponse {
    images: Vec<ImageInfo>,
}

#[derive(Deserialize)]
struct ImageInfo {
    id: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

impl NekosMoeClient {
    pub fn new(
        proxy: Option<&str>,
        token_path: Option<std::path::PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));

        if let Some(ref path) = token_path
            && path.exists()
        {
            let token = std::fs::read_to_string(path)?;
            let token = token.trim();
            if !token.is_empty() {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(token)
                        .map_err(|e| format!("Invalid token in file: {e}"))?,
                );
            }
        }

        let mut builder = reqwest::Client::builder().default_headers(headers);

        if let Some(proxy_url) = proxy {
            builder = builder.proxy(
                reqwest::Proxy::all(proxy_url)
                    .map_err(|e| format!("Failed to configure proxy: {e}"))?,
            );
        }

        let client = builder
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

        Ok(NekosMoeClient { client })
    }

    pub async fn random_images(
        &self,
        nsfw: bool,
        count: u32,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if count == 0 {
            return Err("count must be greater than 0".into());
        }

        let url = format!("{BASE_URL}/random/image?nsfw={nsfw}&count={count}");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch random images: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("API returned status {status}").into());
        }

        let body: RandomImagesResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {e}"))?;

        let image_urls: Vec<String> = body
            .images
            .into_iter()
            .map(|img| format!("https://nekos.moe/image/{}.jpg", img.id))
            .collect();

        Ok(image_urls)
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if username.is_empty() {
            return Err("username must not be empty".into());
        }
        if password.is_empty() {
            return Err("password must not be empty".into());
        }

        let url = format!("{BASE_URL}/auth");

        let payload = serde_json::json!({
            "username": username,
            "password": password,
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Login request failed: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Login failed with status {status}").into());
        }

        let body: LoginResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse login response: {e}"))?;

        Ok(body.token)
    }
}
