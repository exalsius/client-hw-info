mod hardware;

use argh::FromArgs;
use dotenvy::from_path;
use hardware::NodeHardware;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::{env, fs};

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
    dotenvy::dotenv().ok();
    let cli_arguments: CliArguments = argh::from_env();

    let (node_id, api_endpoint, refresh_tkn, auth0_client_id, auth_client_domain) =
        match lookup_configuration(
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
                eprintln!("Error: {}", e);
                return;
            }
        };

    let auth_token = match get_fresh_auth_token(&refresh_tkn, &auth0_client_id, &auth_client_domain)
    {
        Ok(token) => token,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };


    let node_hardware = match hardware::collect_client_hardware() {
        Ok(node_hardware) => node_hardware,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };



    send_heartbeat(&node_id, &api_endpoint, &auth_token, &node_hardware);
}



fn get_fresh_auth_token(
    refresh_token: &str,
    auth0_client_id: &str,
    auth0_client_domain: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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
                Ok(parsed.access_token)
            } else {
                let status_code = resp.status();
                let body = resp
                    .text()
                    .unwrap_or_else(|_| "Unable to read body".to_string());
                Err(format!("Request failed with status {}: {}", status_code, body).into())
            }
        }
        Err(e) => Err(e.into()),
    }
}

fn send_heartbeat(node_id: &str, api_url: &str, auth_token: &str, node_hardware: &NodeHardware) {
    let client = reqwest::blocking::Client::new();
    let final_endpoint = api_url.to_string() + "/node/" + node_id;
    let resp = client
        .patch(final_endpoint)
        .json(&node_hardware)
        .bearer_auth(auth_token)
        .send();

    match resp {
        Ok(resp) => {
            println!("Response: {}", resp.status());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("HOME not set")?;
    let dir = home.join(".config").join("exalsius");
    let file = dir.join("config.env");
    // Verzeichnis sicherstellen
    fs::create_dir_all(&dir)?;
    Ok(file)
}

/// Falls Datei fehlt: neu anlegen (0600) und Template reinschreiben
fn ensure_config_file(
    path: &PathBuf,
    node_id: Option<&str>,
    api_url: Option<&str>,
    refresh_token: Option<&str>,
    auth0_client_id: Option<&str>,
    auth0_client_domain: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        let mut opts = OpenOptions::new();
        opts.create(true).write(true).truncate(true);
        #[cfg(unix)]
        {
            // 0600: nur Besitzer darf lesen/schreiben
            opts.mode(0o600);
        }

        if node_id.is_none() | api_url.is_none() | refresh_token.is_none() | auth0_client_id.is_none() | auth0_client_domain.is_none() {
            return Err(
                "node_id, api_url and auth_token, auth0_client_id, and auth0_client_domain must be given if no environment exist".into(),
            );
        }

        let mut f = opts.open(path)?;
        writeln!(f, "NODE_ID={}", node_id.unwrap())?;
        writeln!(f, "API_URL={}", api_url.unwrap())?;
        writeln!(f, "AUTH_TOKEN={}", refresh_token.unwrap().trim())?;
        writeln!(
            f,
            "AUTH0_CLIENT_DOMAIN={}",
            auth0_client_domain.unwrap().trim()
        )?;
        writeln!(f, "AUTH0_CLIENT_ID={}", auth0_client_id.unwrap().trim())?;
    }
    Ok(())
}

fn lookup_configuration(
    node_id: Option<String>,
    api_url: Option<String>,
    auth_token: Option<String>,
    auth0_client_id: Option<String>,
    auth0_client_domain: Option<String>,
) -> Result<(String, String, String, String, String), Box<dyn std::error::Error>> {
    let cfg = config_file_path()?;
    ensure_config_file(
        &cfg,
        node_id.as_deref(),
        api_url.as_deref(),
        auth_token.as_deref(),
        auth0_client_id.as_deref(),
        auth0_client_domain.as_deref(),
    )?;

    from_path(&cfg)?;

    let api_url = env::var("API_URL")?;
    let auth_token = env::var("AUTH_TOKEN")?;
    let node_id = env::var("NODE_ID")?;
    let auth0_client_id = env::var("AUTH0_CLIENT_ID")?;
    let auth0_client_domain = env::var("AUTH0_CLIENT_DOMAIN")?;

    if api_url.is_empty() || auth_token.is_empty() {
        return Err("API_URL or AUTH_TOKEN is empty".into());
    }

    Ok((
        node_id,
        api_url,
        auth_token,
        auth0_client_id,
        auth0_client_domain,
    ))
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
