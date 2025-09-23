
mod config;
mod hardware;
mod heartbeat;

use argh::FromArgs;
use env_logger::{Builder, Env};
use hardware::NodeHardware;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

#[derive(FromArgs)]
///   Parameters for the client hardware info tool.
struct CliArguments {
    /// the refresh token for acquiring the access token.
    #[argh(option)]
    access_token: Option<String>,

    /// the server API endpoint.
    #[argh(option)]
    api_url: Option<String>,

    /// the id of the node where the tool is running.
    #[argh(option)]
    node_id: Option<String>,

    /// you can skip the heartbeat sending to only run the hardware identification.
    #[argh(option)]
    skip_heartbeat: Option<bool>,
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting client hardware info tool");

    let cli_arguments: CliArguments = argh::from_env();

    let node_hardware = match hardware::collect_client_hardware() {
        Ok(node_hardware) => node_hardware,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    if cli_arguments.skip_heartbeat.unwrap_or(false) {
        info!("Hardware collected (heartbeat skipped by flag)");
        return;
    }

    let (node_id, api_endpoint, auth_tkn) = match config::lookup_configuration(
        cli_arguments.node_id,
        cli_arguments.api_url,
        cli_arguments.access_token,
    ) {
        Ok((node_id, api_url, auth_token)) => (node_id, api_url, auth_token),
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    let new_auth_tkn = match heartbeat::send_heartbeat(&node_id, &api_endpoint, &auth_tkn, &node_hardware) {
        Ok(new_auth_tkn) => new_auth_tkn,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    config::write_new_auth_token(&new_auth_tkn).expect("Failed writing new auth token to config file");

    info!("Finished client hardware info tool");
}

