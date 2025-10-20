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

    let env_api_url = env::var("API_URL")?;
    let env_auth_token = env::var("AUTH_TOKEN")?;
    let env_node_id = env::var("NODE_ID")?;

    if env_api_url.is_empty() || env_auth_token.is_empty() || env_node_id.is_empty() {
        error!("API_URL, AUTH_TOKEN, NODE_ID must not be empty");
        return Err("API_URL or AUTH_TOKEN is empty".into());
    }

    if node_id.as_ref().map_or(false, |id| id != &env_node_id)
        || api_url.as_ref().map_or(false, |url| url != &env_api_url)
        || auth_token.as_ref().map_or(false, |token| token != &env_auth_token)
    {
        info!("Configuration file does not match passed variables. Updating configuration file");
        update_config_file(&cfg, &env_node_id, &env_api_url, &env_auth_token, node_id.as_deref(), api_url.as_deref(), auth_token.as_deref())?;
    }

    info!("Successfully loaded configuration from file");
    Ok((env_node_id, env_api_url, env_auth_token))
}

fn update_config_file(
    config_file_path: &PathBuf,
    env_node_id: &str,
    env_api_url: &str,
    env_auth_token: &str,
    node_id: Option<&str>,
    api_url: Option<&str>,
    auth_token: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {

    let content = fs::read_to_string(config_file_path)?;
    let mut lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if line.starts_with("NODE_ID=") {
            lines.push(format!("NODE_ID={}", node_id.unwrap_or(env_node_id)));
        } else if line.starts_with("API_URL=") {
            lines.push(format!("API_URL={}", api_url.unwrap_or(env_api_url)));
        } else if line.starts_with("AUTH_TOKEN=") {
            lines.push(format!("AUTH_TOKEN={}", auth_token.unwrap_or(env_auth_token)));
        } else {
            lines.push(line.to_string());
        }
    }

    fs::write(config_file_path, lines.join("\n"))?;
    info!("Successfully updated configuration file");
    Ok(())

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
    for line in cfg_contents.lines() {
        if line.starts_with("AUTH_TOKEN=") {
            new_lines.push(format!("AUTH_TOKEN={}", new_auth_token));
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
