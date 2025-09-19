mod auth;
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
    refresh_token: Option<String>,

    /// the server API endpoint.
    #[argh(option)]
    api_url: Option<String>,

    /// the id of the node where the tool is running.
    #[argh(option)]
    node_id: Option<String>,

    /// the auth0 client id.
    #[argh(option)]
    auth0_client_id: Option<String>,

    /// the auth0 client domain.
    #[argh(option)]
    auth0_client_domain: Option<String>,
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting client hardware info tool");

    let cli_arguments: CliArguments = argh::from_env();

    let (node_id, api_endpoint, refresh_tkn, auth0_client_id, auth_client_domain) =
        match config::lookup_configuration(
            cli_arguments.node_id,
            cli_arguments.api_url,
            cli_arguments.refresh_token,
            cli_arguments.auth0_client_id,
            cli_arguments.auth0_client_domain,
        ) {
            Ok((node_id, api_url, auth_token, auth0_client_id, auth0_client_domain)) => (
                node_id,
                api_url,
                auth_token,
                auth0_client_id,
                auth0_client_domain,
            ),
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

    let auth_token =
        match auth::get_fresh_auth_token(&refresh_tkn, &auth0_client_id, &auth_client_domain) {
            Ok(token) => token,
            Err(e) => {
                error!("Auth token refresh failed: {e:#}");
                return;
            }
        };

    let node_hardware = match hardware::collect_client_hardware() {
        Ok(node_hardware) => node_hardware,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    heartbeat::send_heartbeat(&node_id, &api_endpoint, &auth_token, &node_hardware);
    info!("Finished client hardware info tool");
}
