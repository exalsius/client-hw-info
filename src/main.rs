use argh::FromArgs;
use dotenvy::from_path;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
use sysinfo::{Disks, System};

#[derive(FromArgs)]
///   Parameters for the client hardware info tool.
struct CliArguments {
    /// the auth token for sending a heartbeat to the API.
    #[argh(option)]
    auth_token: Option<String>,

    /// the server API endpoint.
    #[argh(option)]
    api_url: Option<String>,

    /// the id of the node where the tool is running.
    #[argh(option)]
    node_id: Option<String>,
}

fn main() {
    dotenvy::dotenv().ok();
    let cli_arguments: CliArguments = argh::from_env();

    let (node_id, api_endpoint, auth_tkn) = match lookup_auth_token(
        cli_arguments.node_id,
        cli_arguments.api_url,
        cli_arguments.auth_token,
    ) {
        Ok((node_id, api_url, auth_token)) => (node_id, api_url, auth_token),
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let mut node_hardware = NodeHardware {
        gpu_count: 0,
        gpu_vendor: String::from("unknown"),
        gpu_type: String::from("unknown"),
        gpu_memory: 0,
        cpu_cores: 0,
        memory_gb: 0,
        storage_gb: 0,
    };

    let mut sys = System::new_all();
    sys.refresh_all();

    println!("Total memory: {} GiB", bytes_to_gib(sys.total_memory()));
    node_hardware.memory_gb = bytes_to_gib(sys.total_memory());
    println!("Total number of CPU threads: {}", sys.cpus().len());
    node_hardware.cpu_cores = sys.cpus().len() as u64;

    let disks = Disks::new_with_refreshed_list();
    if let Some(disk) = disks
        .iter()
        .find(|disk| disk.mount_point() == Path::new("/"))
    {
        println!(
            "Root disk with filesystem {} and {} GB storage",
            disk.file_system().to_string_lossy(),
            bytes_to_gb(disk.total_space())
        );
        node_hardware.storage_gb = bytes_to_gb(disk.total_space());
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(r#"lspci -nn | egrep -i 'vga|3d|display'"#)
        .output()
        .expect("Exception running lspci");

    if !output.status.success() {
        eprintln!("lspci failed");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("GPUs:");
    for line in stdout.lines() {
        if let Some(start) = line.rfind('[') {
            if let Some(end) = line[start + 1..].find(']') {
                let id_str = &line[start + 1..start + 1 + end];
                let parts: Vec<&str> = id_str.split(":").collect();
                let vendor = lookup_vendor(parts[0]);
                let (gpu_name, vram) = lookup_device(parts[1]);

                if let Some(v) = vendor {
                    println!(
                        "--- Vendor = {}, Model = {} with {} GB of VRAM",
                        v, gpu_name, vram
                    );
                    node_hardware.gpu_vendor = v.to_owned();
                    node_hardware.gpu_type = gpu_name.to_owned();
                    node_hardware.gpu_memory = vram as u64;
                    node_hardware.gpu_count += 1;
                }
            }
        }
    }

    send_heartbeat(&node_id, &api_endpoint, &auth_tkn, &node_hardware);
}

fn bytes_to_gb(bytes: u64) -> u64 {
    bytes / (1000 * 1000 * 1000)
}

fn bytes_to_gib(bytes: u64) -> u64 {
    bytes / (1024 * 1024 * 1024)
}

fn lookup_vendor(vendor_id: &str) -> Option<&str> {
    match vendor_id {
        "10de" => Some("NVIDIA"),
        "1002" => Some("AMD"),
        _ => None,
    }
}

fn lookup_device(device_id: &str) -> (&str, u16) {
    match device_id {
        "27b8" => ("L4", 24),
        "26b5" => ("L40", 48),
        "26b9" => ("L40S", 48),
        "20b0" => ("A100", 40),
        "20b1" => ("A100", 40),
        "20b2" => ("A100", 80),
        "20b3" => ("A100", 80),
        "740f" => ("MI210", 64),
        _ => ("unknown device", 0),
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
    auth_token: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        let mut opts = OpenOptions::new();
        opts.create(true).write(true).truncate(true);
        #[cfg(unix)]
        {
            // 0600: nur Besitzer darf lesen/schreiben
            opts.mode(0o600);
        }

        if node_id.is_none() | api_url.is_none() | auth_token.is_none() {
            return Err(
                "node_id, api_url and auth_token must be given if no environment exist".into(),
            );
        }

        let mut f = opts.open(path)?;
        writeln!(f, "NODE_ID={}", node_id.unwrap())?;
        writeln!(f, "API_URL={}", api_url.unwrap())?;
        writeln!(f, "AUTH_TOKEN={}", auth_token.unwrap().trim())?;
    }
    Ok(())
}


fn lookup_auth_token(
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

    from_path(&cfg)?;

    let api_url = env::var("API_URL")?;
    let auth_token = env::var("AUTH_TOKEN")?;
    let node_id = env::var("NODE_ID")?;

    if api_url.is_empty() || auth_token.is_empty() {
        return Err("API_URL or AUTH_TOKEN is empty".into());
    }

    Ok((node_id, api_url, auth_token))
}

#[derive(Serialize, Debug)]
struct NodeHardware {
    gpu_count: u8,
    gpu_vendor: String,
    gpu_type: String,
    gpu_memory: u64,
    cpu_cores: u64,
    memory_gb: u64,
    storage_gb: u64,
}
