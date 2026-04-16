use crate::config;
use crate::hardware::NodeHardware;
use crate::software::NodeSoftware;
use crate::system::NodeSystem;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

const SYSTEMD_SERVICE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/systemd/client-hw-info.service"
));

const SYSTEMD_TIMER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/systemd/client-hw-info.timer"
));

#[derive(Serialize)]
struct SelfRegisterRequest<'a> {
    register_token: &'a str,
    hardware: &'a NodeHardware,
    software: &'a NodeSoftware,
    system: &'a NodeSystem,
    username: &'a str,
    ssh_key_id: &'a str,
    hostname: &'a str,
    endpoint: &'a String,
    price_per_hour: f64,
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
    ssh_key_id: &str,
    hostname: &str,
    ip_addr: &String,
    port: u16,
    price_per_hour: f64,
    skip_systemd: bool,
) -> Result<SelfRegisterResponse, Box<dyn std::error::Error>> {

    let cfg_path = crate::config::config_file_path()?;

    self_register_with_config_path(
        api_url,
        register_token,
        node_hardware,
        node_software,
        node_system,
        username,
        ssh_key_id,
        hostname,
        ip_addr,
        port,
        price_per_hour,
        skip_systemd,
        &cfg_path,
    )

}

fn self_register_with_config_path(
    api_url: &str,
    register_token: &str,
    node_hardware: &NodeHardware,
    node_software: &NodeSoftware,
    node_system: &NodeSystem,
    username: &str,
    ssh_key_id: &str,
    hostname: &str,
    ip_addr: &String,
    port: u16,
    price_per_hour: f64,
    skip_systemd: bool,
    cfg_path: &PathBuf,
) -> Result<SelfRegisterResponse, Box<dyn std::error::Error>> {

    let client = reqwest::blocking::Client::new();

    let final_endpoint = format!("{}/node/self-register", api_url.trim_end_matches('/'));

    let payload = SelfRegisterRequest {
        register_token,
        hardware: node_hardware,
        software: node_software,
        system: node_system,
        ssh_key_id,
        username,
        hostname,
        endpoint: &format!("{}:{}", ip_addr, port),
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

        match config::create_config_file(
            &cfg_path,
            &parsed.node_id,
            api_url,
            &parsed.next_access_token,
        ) {
            Ok(_) => {
                info!("Successfully created new configuration file for newly registered node");
            }
            Err(e) => {
                error!(
                    "Failed creating new configuration file for newly registered node: {}",
                    e
                );
                return Err(e);
            }
        }

        if !skip_systemd {
            create_systemd_service()?;
            create_systemd_timer(15)?;
            reload_and_enable_timer()?;
        }

        Ok(parsed)
    } else {
        warn!("Self-register request failed with status {}", resp.status());
        Err(format!("self-register failed with status {}", resp.status()).into())
    }


}

fn create_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating systemd service for node");

    let current_binary_path: PathBuf = env::current_exe()?;
    let current_binary_path_string = current_binary_path.display().to_string();

    let rendered = SYSTEMD_SERVICE_TEMPLATE.replace("{{EXEC_START}}", &current_binary_path_string);

    let service_path = Path::new("/etc/systemd/system/client-hw-info.service");
    fs::write(service_path, rendered)?;
    Ok(())
}

fn create_systemd_timer(heartbeat_interval: u8) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating systemd timer for node");

    let rendered = SYSTEMD_TIMER_TEMPLATE.replace(
        "{{HEARTBEAT_INTERVAL_MINUTES}}",
        &heartbeat_interval.to_string(),
    );

    let timer_path = Path::new("/etc/systemd/system/client-hw-info.timer");

    fs::write(timer_path, rendered)?;

    Ok(())
}

fn reload_and_enable_timer() -> Result<(), Box<dyn std::error::Error>> {
    let reload_status = Command::new("systemctl").arg("daemon-reload").status()?;

    if !reload_status.success() {
        return Err("systemctl daemon-reload failed".into());
    }

    let enable_status = Command::new("systemctl")
        .args(["enable", "--now", "client-hw-info.timer"])
        .status()?;

    if !enable_status.success() {
        return Err("systemctl enable --now client-hw-info.timer failed".into());
    }

    Ok(())
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

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let cfg_path = temp_dir.path().join("config.env");

        println!("temp config path: {}", cfg_path.display());

        let mut server = Server::new();

        let _mock = server
            .mock("POST", "/node/self-register")
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
        let hostname = "node-1";
        let port = 22;
        let ip_addr = "127.0.0.1".to_string();
        let price_per_hour = 1.25;
        let skip_systemd = true;

        let result = self_register_with_config_path(
            &server.url(),
            register_token,
            &hardware,
            &software,
            &system,
            username,
            private_key_id,
            hostname,
            &ip_addr,
            port,
            price_per_hour,
            skip_systemd,
            &cfg_path,
        );

        let config = std::fs::read_to_string(&cfg_path).expect("config file should be written");

        assert!(config.contains("NODE_ID=node-123"));
        assert!(config.contains("AUTH_TOKEN=token-abc"));
        assert!(config.contains(&format!("API_URL={}", server.url())));

        let res_unwrap = result.expect("self_register should succeed");

        assert_eq!(res_unwrap.node_id, "node-123");
        assert_eq!(res_unwrap.next_access_token, "token-abc");
    }
}
