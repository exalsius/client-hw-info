use log::{error, info, warn};
use serde::Deserialize;
use crate::hardware::NodeHardware;

pub(crate) fn send_heartbeat(node_id: &str, api_url: &str, auth_token: &str, node_hardware: &NodeHardware) -> Result<String, Box<dyn std::error::Error>> {
    info!("Sending heartbeat");
    let client = reqwest::blocking::Client::new();
    let final_endpoint = api_url.to_string() + "/node/" + node_id;
    let resp = client
        .patch(final_endpoint)
        .json(&node_hardware)
        .bearer_auth(auth_token)
        .send();

    match resp {
        Ok(resp) => {
            if resp.status().is_success() {
                let parsed = resp.json::<HeartbeatResponse>().map_err(|e| {
                    error!("Failed parsing server response: {}", e); e
                })?;

                info!("Successfully sent heartbeat and patched node hardware");
                Ok(parsed.next_access_token)
            } else {
                warn!("Heartbeat went through but server responded with status code {}", resp.status());
                Err(format!("heartbeat failed with status {}", resp.status()).into())
            }
        }
        Err(e) => {
            Err(e.into())
        }

    }

}


#[derive(Deserialize, Debug)]
struct HeartbeatResponse {
    node_id: String,
    next_access_token: String,
    next_access_token_expires_in: u64,
    next_access_token_type: String
}