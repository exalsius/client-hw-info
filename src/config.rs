use dotenvy::from_path;
use log::{error, info};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::{env, fs, io};

fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("HOME not set")?;
    let dir = home.join(".config").join("exalsius");
    let file = dir.join("config.env");

    fs::create_dir_all(&dir)?;
    Ok(file)
}

pub(crate) fn lookup_configuration(
    node_id: Option<String>,
    api_url: Option<String>,
    auth_token: Option<String>,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let cfg = config_file_path()?;
    ensure_config_file(
        &cfg,
        node_id.as_deref(),
        api_url.as_deref(),
        auth_token.as_deref(),
    )?;

    info!("Loading configuration from file");
    from_path(&cfg)?;

    let api_url = env::var("API_URL")?;
    let auth_token = env::var("AUTH_TOKEN")?;
    let node_id = env::var("NODE_ID")?;

    if api_url.is_empty() || auth_token.is_empty() || node_id.is_empty() {
        error!("API_URL, AUTH_TOKEN, NODE_ID must not be empty");
        return Err("API_URL or AUTH_TOKEN is empty".into());
    }
    info!("Successfully loaded configuration from file");
    Ok((node_id, api_url, auth_token))
}

/// Falls Datei fehlt: neu anlegen (0600) und Template reinschreiben
fn ensure_config_file(
    path: &PathBuf,
    node_id: Option<&str>,
    api_url: Option<&str>,
    auth_token: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        if node_id.is_none() || api_url.is_none() || auth_token.is_none() {
            return Err(
                "node_id, api_url and auth_token must be given if no environment exist".into(),
            );
        }

        create_config_file(
            &path,
            node_id.unwrap(),
            api_url.unwrap(),
            auth_token.unwrap(),
        )?;
    } else {
        info!("Configuration file already exists. Skipping creation");
    }

    Ok(())
}

fn create_config_file(
    path: &PathBuf,
    node_id: &str,
    api_url: &str,
    auth_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Start creating new configuration file");
    let mut opts = OpenOptions::new();
    opts.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        // 0600: nur Besitzer darf lesen/schreiben
        opts.mode(0o600);
    }

    let mut f = opts.open(path)?; // handle Result properly
    writeln!(f, "NODE_ID={}", node_id).map_err(|e| {
        error!("Failed writing NODE_ID to {}: {e}", path.display());
        e
    })?;
    writeln!(f, "API_URL={}", api_url).map_err(|e| {
        error!("Failed writing API_URL to {}: {e}", path.display());
        e
    })?;
    writeln!(f, "AUTH_TOKEN={}", auth_token.trim()).map_err(|e| {
        error!("Failed writing AUTH_TOKEN to {}: {e}", path.display());
        e
    })?;
    info!("Created new configuration file");
    Ok(())
}

pub fn write_new_auth_token(new_auth_token: &String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Writing new auth token to config file");
    let cfg_path = config_file_path()?;

    let cfg_contents = fs::read_to_string(&cfg_path)?;

    let mut new_lines = Vec::new();
    let mut replaced = false;
    for line in cfg_contents.lines() {
        if line.starts_with("AUTH_TOKEN=") {
            new_lines.push(format!("AUTH_TOKEN={}", new_auth_token));
            replaced = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    let mut file = fs::File::create(&cfg_path).map_err(|e| {
        error!("Failed overriding config file: {}", e);
        e
    })?;

    file.write_all(new_lines.join("\n").as_bytes())?;
    info!("Successfully wrote new auth token to config file");
    Ok(())
}
