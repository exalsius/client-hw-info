mod config;
mod hardware;
mod heartbeat;
mod self_register;
mod software;
mod system;

use argh::FromArgs;
use env_logger::{Builder, Env};
use log::{error, info};

#[derive(FromArgs)]
///   Parameters for the client hardware info tool.
struct CliArguments {
    /// print the version and exit.
    #[argh(switch, short = 'V')]
    version: bool,

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
    #[argh(switch)]
    skip_heartbeat: bool,

    /// start self-registering workflow. Only works once.
    #[argh(switch)]
    self_register: bool,

    /// the register token of the self-registering node that can be retrieved in the user profile
    #[argh(option)]
    register_token: Option<String>,

    /// the node name of the self-registering node
    #[argh(option)]
    hostname: Option<String>,

    /// the ip address of the self-registering node
    #[argh(option)]
    ip_addr: Option<String>,

    /// the port of the self-registering node
    #[argh(option)]
    port: Option<u16>,

    /// the username of the self-registering node
    #[argh(option)]
    username: Option<String>,

    /// the private key id of the self-registering node
    #[argh(option)]
    private_key_id: Option<String>,

    /// skip systemd service creation when running self-registering
    #[argh(switch)]
    skip_systemd: bool,
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli_arguments: CliArguments = argh::from_env();

    if cli_arguments.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"),);
        return;
    }

    info!("Starting client hardware info tool");

    let node_hardware = match hardware::collect_client_hardware() {
        Ok(node_hardware) => node_hardware,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    let node_software = software::collect_software_info();

    let node_system = system::collect_system_info();

    if cli_arguments.skip_heartbeat {
        info!("Hardware, Software, and OS details collected (heartbeat skipped by flag)");
        return;
    }

    if cli_arguments.self_register {
        if cli_arguments.register_token.is_none()
            || cli_arguments.username.is_none()
            || cli_arguments.private_key_id.is_none()
            || cli_arguments.hostname.is_none()
            || cli_arguments.ip_addr.is_none()
            || cli_arguments.port.is_none()
            || cli_arguments.api_url.is_none()
        {
            error!(
                "Error: self-registering requires register token, username, private key id, node name, ip address, port, and api url"
            );
            return;
        }

        info!("Starting self-registering process");

        let api_url = cli_arguments.api_url.unwrap();
        let username = cli_arguments.username.unwrap();
        let private_key_id = cli_arguments.private_key_id.unwrap();
        let register_token = cli_arguments.register_token.unwrap();
        let hostname = cli_arguments.hostname.unwrap();
        let ip_addr = cli_arguments.ip_addr.unwrap();
        let port = cli_arguments.port.unwrap();
        let skip_systemd = cli_arguments.skip_systemd;

        match self_register::self_register(
            &api_url,
            &register_token,
            &node_hardware,
            &node_software,
            &node_system,
            &username,
            &private_key_id,
            &hostname,
            &ip_addr,
            port,
            0.0,
            skip_systemd,
        ) {
            Ok(_) => {
                info!("Successfully registered node");
                return;
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }
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

    let new_auth_tkn = match heartbeat::send_heartbeat(
        &node_id,
        &api_endpoint,
        &auth_tkn,
        &node_hardware,
        &node_software,
        &node_system,
    ) {
        Ok(new_auth_tkn) => new_auth_tkn,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    config::write_new_auth_token(&new_auth_tkn)
        .expect("Failed writing new auth token to config file");

    info!("Finished client hardware info tool");
}
