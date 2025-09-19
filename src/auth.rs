use log::{error, info, warn};
use serde::{Deserialize, Serialize};

pub(crate) fn get_fresh_auth_token(
    refresh_token: &str,
    auth0_client_id: &str,
    auth0_client_domain: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Requesting fresh auth token");
    let client = reqwest::blocking::Client::new();
    let final_endpoint = String::from("https://") + auth0_client_domain + "/oauth/token";

    let token_refresh_rq = RefresTokenRequest {
        grant_type: String::from("refresh_token"),
        client_id: auth0_client_id.to_string(),
        refresh_token: refresh_token.to_string(),
        scope: String::from("openid offline_access nodeagent")
    };

    let resp = client.post(final_endpoint).json(&token_refresh_rq).send();

    match resp {
        Ok(resp) => {
            if resp.status().is_success() {
                let parsed = resp.json::<RefreshTokenResponse>()?;
                info!("Received fresh auth token");
                Ok(parsed.access_token)
            } else {
                let status_code = resp.status();
                let body = resp
                    .text()
                    .unwrap_or_else(|_| "Unable to read body".to_string());
                warn!("Request failed with status {}: {}", status_code, body);
                Err(format!("Request failed with status {}: {}", status_code, body).into())
            }
        }
        Err(e) => {
            warn!("Error sending request: {}", e);
            Err(e.into())
        },
    }
}

#[derive(Serialize, Debug)]
struct RefresTokenRequest {
    grant_type: String,
    client_id: String,
    refresh_token: String,
    scope: String,
}

#[derive(Deserialize, Debug)]
struct RefreshTokenResponse {
    access_token: String,
    id_token: String,
    scope: String,
    token_type: String,
    expires_in: u64,
}