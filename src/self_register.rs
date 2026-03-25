use crate::hardware::NodeHardware;
use crate::software::NodeSoftware;
use crate::system::NodeSystem;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use crate::config;

#[derive(Serialize)]
struct SelfRegisterRequest<'a> {
    register_token: &'a str,
    hardware: &'a NodeHardware,
    software: &'a NodeSoftware,
    system: &'a NodeSystem,
    username: &'a str,
    private_key_id: &'a str,
    node_name: &'a str,
    port: &'a u16,
    ip_addr: &'a String,
    price_per_hour: &'a f64,
}
#[derive(Deserialize, Debug)]
pub(crate) struct SelfRegisterResponse {
    node_id: String,
    next_access_token: String,
}

pub(crate) fn self_register(
    api_url: &str,
    register_token: &str,
    node_hardware: &NodeHardware,
    node_software: &NodeSoftware,
    node_system: &NodeSystem,
    username: &str,
    private_key_id: &str,
    node_name: &str,
    ip_addr: &String,
    port: &u16,
    price_per_hour: &f64,
) -> Result<SelfRegisterResponse, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let final_endpoint = api_url.to_string() + "/self-register";

    let payload = SelfRegisterRequest {
        register_token,
        hardware: node_hardware,
        software: node_software,
        system: node_system,
        private_key_id,
        username,
        node_name,
        port,
        ip_addr,
        price_per_hour,
    };
    info!("Sending self-register request to {}", final_endpoint);
    let resp = client.post(final_endpoint).json(&payload).send()?;

    if resp.status().is_success() {
        info!("Successfully sent self-register request. Parsing response.");
        let parsed = resp.json::<SelfRegisterResponse>().map_err(|e| {
            error!("Failed to parse self-register response: {}", e);
            e
        })?;

        info!("Successfully parsed self-register response. Writing new configuration file.");
        let cfg_path = crate::config::config_file_path()?;

        match config::create_config_file(&cfg_path, &parsed.node_id, api_url, &parsed.next_access_token) {
            Ok(_) => {
                info!("Successfully created new configuration file for newly registered node");
                Ok(parsed)
            }
            Err(e) => {
                error!("Failed creating new configuration file for newly registered node: {}", e);
                Err(e)
            }
        }

    } else {
        warn!("Self-register request failed with status {}", resp.status());
        Err(format!("self-register failed with status {}", resp.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn create_mock_hardware() -> NodeHardware {
        NodeHardware {
            gpu_count: 1,
            gpu_vendor: String::from("Nvidia"),
            gpu_type: String::from("AD102GL [L40]"),
            gpu_memory: 48,
            cpu_cores: 16,
            memory_gb: 64,
            storage_gb: 1024,
        }
    }

    fn create_mock_software() -> NodeSoftware {
        NodeSoftware {
            docker: String::from(""),
            nvidia: String::from(""),
            amd: String::from(""),
        }
    }

    fn create_mock_system() -> NodeSystem {
        NodeSystem {
            os: String::from("Linux (Ubuntu 24.04)"),
            kernel: String::from("Linux 6.11.0-26-generic"),
        }
    }

    #[test]
    fn test_self_register_success() {
        let mut server = Server::new();

        let _mock = server
            .mock("POST", "/self-register")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "node_id": "node-123",
                    "next_access_token": "token-abc",
                    "next_access_token_expires_in": 3600,
                    "next_access_token_type": "Bearer"
                }"#,
            )
            .create();

        let hardware = create_mock_hardware();
        let software = create_mock_software();
        let system = create_mock_system();

        let username = "ubuntu";
        let register_token = "TOKEN_IN_USER_PROFILE";
        let private_key_id = "PRIVATE_KEY_TO_ACCESS_NODE";
        let node_name = "node-1";
        let port = 22;
        let ip_addr = "127.0.0.1".to_string();
        let price_per_hour = 1.25;

        let result = self_register(
            &server.url(),
            register_token,
            &hardware,
            &software,
            &system,
            private_key_id,
            username,
            node_name,
            &ip_addr,
            &port,
            &price_per_hour,
        );

        assert!(result.is_ok());

        let res_unwrap = result.unwrap();

        assert_eq!(res_unwrap.node_id, "node-123");
        assert_eq!(res_unwrap.next_access_token, "token-abc");


    }
}
