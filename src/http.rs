use reqwest::Client;
use std::env;

pub async fn tradier_post(uri: &str, payload: Option<String>) -> Result<String, reqwest::Error> {
    let api_key = env::var("TRADIER_API_KEY")
        .expect("Required TRADIER_API_KEY environment variable was not found");
    const BASE_URL: &str = "https://api.tradier.com/v1";
    let url = [BASE_URL, uri].concat();

    let client = Client::new();
    let body = payload.unwrap_or_default();
    let content_length = body.len();

    client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/json")
        .header("Content-Length", content_length)
        .body(body)
        .send()
        .await?
        .text()
        .await
}

pub async fn tradier_get(uri: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("TRADIER_API_KEY")
        .expect("Required TRADIER_API_KEY environment variable was not found");
    const BASE_URL: &str = "https://api.tradier.com/v1";
    let url = [BASE_URL, uri].concat();

    let client = Client::new();

    client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/json")
        .send()
        .await?
        .text()
        .await
}
