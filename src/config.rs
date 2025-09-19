use dotenvy::from_path;
use log::{error, info};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::{env, fs};

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
        info!("Start creating new configuration file");
        let mut opts = OpenOptions::new();
        opts.create(true).write(true).truncate(true);
        #[cfg(unix)]
        {
            // 0600: nur Besitzer darf lesen/schreiben
            opts.mode(0o600);
        }

        if node_id.is_none()
            | api_url.is_none()
            | refresh_token.is_none()
            | auth0_client_id.is_none()
            | auth0_client_domain.is_none()
        {
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
        info!("Created new configuration file")
    } else {
        info!("Configuration file already exists. Skipping creation")
    }

    Ok(())
}

pub(crate) fn lookup_configuration(
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

    info!("Loading configuration from file");
    from_path(&cfg)?;

    let api_url = env::var("API_URL")?;
    let auth_token = env::var("AUTH_TOKEN")?;
    let node_id = env::var("NODE_ID")?;
    let auth0_client_id = env::var("AUTH0_CLIENT_ID")?;
    let auth0_client_domain = env::var("AUTH0_CLIENT_DOMAIN")?;

    if api_url.is_empty()
        || auth_token.is_empty()
        || node_id.is_empty()
        || auth0_client_id.is_empty()
        || auth0_client_domain.is_empty()
    {
        error!(
            "API_URL, AUTH_TOKEN, NODE_ID, AUTH0_CLIENT_ID, and AUTH0_CLIENT_DOMAIN must not be empty"
        );
        return Err("API_URL or AUTH_TOKEN is empty".into());
    }
    info!("Successfully loaded configuration from file");
    Ok((
        node_id,
        api_url,
        auth_token,
        auth0_client_id,
        auth0_client_domain,
    ))
}
