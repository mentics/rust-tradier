use anyhow::Result;
use reqwest::Client;
use std::env;

pub async fn tradier_post(uri: &str, payload: Option<String>) -> Result<String> {
    let api_key = match env::var("TRADIER_API_KEY") {
        Ok(key) => key,
        Err(_) => anyhow::bail!("TRADIER_API_KEY environment variable not found")
    };
    const BASE_URL: &str = "https://api.tradier.com/v1";
    let url = [BASE_URL, uri].concat();

    let client = Client::new();
    let body = payload.unwrap_or_default();
    let content_length = body.len();

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/json")
        .header("Content-Length", content_length)
        .body(body)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        anyhow::bail!(
            "API request failed with status {}: {}",
            status,
            response_text
        );
    }

    Ok(response_text)
}

pub async fn tradier_get(uri: &str) -> Result<String> {
    tradier_get_versioned(uri, "v1").await
}

pub async fn tradier_get_versioned(uri: &str, version: &str) -> Result<String> {
    let api_key = match env::var("TRADIER_API_KEY") {
        Ok(key) => key,
        Err(_) => anyhow::bail!("TRADIER_API_KEY environment variable not found")
    };
    let url = format!("https://api.tradier.com/{}{}", version, uri);

    let client = Client::new();

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/json")
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;

    if !status.is_success() {
        anyhow::bail!(
            "API request failed with status {}: {}",
            status,
            response_text
        );
    }

    Ok(response_text)
}
