use serde::{Deserialize, Serialize};
use tracing::{warn, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoamiResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub tier: String,
    pub item_limit: Option<i64>,
    pub item_count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub whoami: Option<WhoamiResponse>,
    pub error: Option<String>,
}

fn get_api_config() -> Result<(String, String), String> {
    let config_path = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".candlekeep/config.toml");

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: toml::Value = content
        .parse()
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let api_key = config
        .get("auth")
        .and_then(|auth| auth.get("api_key"))
        .and_then(|key| key.as_str())
        .ok_or("No API key found in config")?
        .to_string();

    let api_url = std::env::var("CANDLEKEEP_API_URL").unwrap_or_else(|_| {
        config
            .get("api")
            .and_then(|api| api.get("url"))
            .and_then(|url| url.as_str())
            .unwrap_or("https://www.getcandlekeep.com")
            .to_string()
    });

    Ok((api_url, api_key))
}

#[tauri::command]
pub async fn fetch_whoami() -> Result<WhoamiResponse, String> {
    let (api_url, api_key) = get_api_config()?;
    let url = format!("{}/api/v1/auth/whoami", api_url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("User-Agent", format!("candlekeep-desktop/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| {
            error!("API request to whoami failed: {}", e);
            format!("Request failed: {}", e)
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!("API returned non-success status {}: {}", status, body);
        return Err(format!("API error ({}): {}", status, body));
    }

    response
        .json::<WhoamiResponse>()
        .await
        .map_err(|e| {
            error!("Failed to parse whoami response: {}", e);
            format!("Failed to parse response: {}", e)
        })
}

#[tauri::command]
pub async fn fetch_metrics() -> Result<Metrics, String> {
    match fetch_whoami().await {
        Ok(whoami) => Ok(Metrics {
            whoami: Some(whoami),
            error: None,
        }),
        Err(e) => Ok(Metrics {
            whoami: None,
            error: Some(e),
        }),
    }
}
