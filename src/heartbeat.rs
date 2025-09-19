use log::{error, info, warn};
use crate::hardware::NodeHardware;

pub(crate) fn send_heartbeat(node_id: &str, api_url: &str, auth_token: &str, node_hardware: &NodeHardware) {
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
                info!("Successfully sent heartbeat and patched node hardware");
            } else {
                warn!("Heartbeat went through but server responded with status code {}", resp.status())
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}