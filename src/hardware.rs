use log::{error, info};
use pciid_parser::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{fs, io};
use sysinfo::{Disks, System};

const GPU_VRAM_TOML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/gpu_vram.toml"));

pub fn collect_client_hardware() -> Result<NodeHardware, Box<dyn std::error::Error>> {
    info!("Start collecting hardware information");
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut node_hardware = NodeHardware {
        gpu_count: 0,
        gpu_vendor: String::from("UNKNOWN"),
        gpu_type: String::from("UNKNOWN"),
        gpu_memory: 0,
        cpu_cores: 0,
        memory_gb: 0,
        storage_gb: 0,
    };
    node_hardware.memory_gb = bytes_to_gib(sys.total_memory());
    info!("Total memory: {} GiB", node_hardware.memory_gb);

    node_hardware.cpu_cores = sys.cpus().len() as u64;
    info!("Total number of CPU cores: {}", node_hardware.cpu_cores);

    let disks = Disks::new_with_refreshed_list();
    if let Some(disk) = disks
        .iter()
        .find(|disk| disk.mount_point() == Path::new("/"))
    {
        node_hardware.storage_gb = bytes_to_gb(disk.total_space());
        info!(
            "Root disk with filesystem {} and {} GB storage",
            disk.file_system().to_string_lossy(),
            node_hardware.storage_gb
        );
    }

    let ethernet_connections = list_ethernet_connections().unwrap();

    for (idx, ethernet_connection) in ethernet_connections.iter().enumerate() {
        info!(
            "Ethernet connection {} with name {} and speed of {} Mbps",
            idx, ethernet_connection.0, ethernet_connection.1
        );
    }

    let gpus = match list_pci_gpus() {
        Ok(gpus) => gpus,
        Err(e) => {
            return Err(e.into());
        }
    };

    if !gpus.is_empty() {
        node_hardware.gpu_count = gpus.len() as u8;
        node_hardware.gpu_vendor = gpus[0].vendor.to_owned();
        node_hardware.gpu_type = gpus[0].gpu_type.to_owned();
        node_hardware.gpu_memory = gpus[0].vram;
    }

    info!("List GPUs:");
    for (idx, gpu) in gpus.iter().enumerate() {
        info!("GPU {idx}: {} {} {} GB", gpu.vendor, gpu.gpu_type, gpu.vram);
    }

    info!("Finished collecting hardware information");
    Ok(node_hardware)
}

fn list_ethernet_connections() -> io::Result<Vec<(String, i32)>> {
    let items = fs::read_dir("/sys/class/net")?
        .filter_map(|entry| {
            let entry = entry.ok()?; // DirEntry oder skip
            let name = entry.file_name().into_string().ok()?;
            if !name.contains("en") {
                return None;
            }

            let ty = fs::read_to_string(entry.path().join("type"))
                .ok()?
                .trim()
                .parse::<i32>()
                .ok()?;
            if ty != 1 {
                return None;
            }

            let speed = fs::read_to_string(entry.path().join("speed"))
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(-1);

            Some((name, speed))
        })
        .collect::<Vec<(String, i32)>>(); // <— wichtig

    Ok(items)
}

fn list_pci_gpus() -> Result<Vec<GPU>, Box<dyn std::error::Error>> {
    let mut all_gpus = Vec::new();


    let pci_db = Database::get_online().unwrap_or_else(|e| {
        error!("Failed fetching online PCI database: {e}");
        info!("Falling back to offline database");
        Database::read().unwrap()
    });


    let gpu_vram_map = load_gpu_vram_map_from_str(GPU_VRAM_TOML)?;

    for entry in fs::read_dir("/sys/bus/pci/devices/")? {
        let pci_entry = entry?;
        let vendor_id = pci_entry.path().join("vendor");
        let device_id = pci_entry.path().join("device");
        let class_code = pci_entry.path().join("class");

        let class = fs::read_to_string(&class_code)
            .map_err(|e| {
                error!("Failed reading the GPU class {e}");
                e
            })?
            .trim()
            .to_string();

        if !class.starts_with("0x03") {
            continue;
        }

        let vendor_string = fs::read_to_string(&vendor_id)
            .map_err(|e| {
                error!("Failed reading the GPU vendor {e}");
                e
            })?
            .trim()
            .to_string();

        let vendor_hex =
            u16::from_str_radix(vendor_string.trim_start_matches("0x"), 16).map_err(|e| {
                error!("Invalid vendor ID {vendor_string}: {e}");
                e
            })?;

        let device_string = fs::read_to_string(&device_id)
            .map_err(|e| {
                error!("Failed reading the GPU device {e}");
                e
            })?
            .trim()
            .to_string();

        let device_hex =
            u16::from_str_radix(device_string.trim_start_matches("0x"), 16).map_err(|e| {
                error!("Invalid vendor ID {device_string}: {e}");
                e
            })?;

        let pci_vendor = pci_db.vendors.get(&vendor_hex);
        let gpu_device = pci_vendor.and_then(|v| v.devices.get(&device_hex));

        let vendor_mapped = pci_vendor.and_then(|v| map_vendor_to_api_enum(&v.name));
        let gpu_vram = gpu_vram_map.get(&device_string);

        if vendor_mapped.is_some() && gpu_device.is_some() {
            all_gpus.push(GPU {
                vendor: vendor_mapped.unwrap(),
                gpu_type: gpu_device.unwrap().name.to_owned(),
                vram: *gpu_vram.unwrap_or(&0),
            })
        }
    }

    Ok(all_gpus)
}

struct GPU {
    vendor: String,
    gpu_type: String,
    vram: u64,
}

#[derive(Serialize, Debug)]
pub struct NodeHardware {
    gpu_count: u8,
    gpu_vendor: String,
    gpu_type: String,
    gpu_memory: u64,
    cpu_cores: u64,
    memory_gb: u64,
    storage_gb: u64,
}

fn bytes_to_gb(bytes: u64) -> u64 {
    bytes / (1000 * 1000 * 1000)
}

fn bytes_to_gib(bytes: u64) -> u64 {
    bytes / (1024 * 1024 * 1024)
}

fn map_vendor_to_api_enum(vendor_name: &str) -> Option<String> {
    match vendor_name {
        "Advanced Micro Devices, Inc. [AMD/ATI]" => Some(String::from("AMD")),
        "NVIDIA Corporation" => Some(String::from("NVIDIA")),
        "Intel Corporation" => Some(String::from("INTEL")),
        _ => None,
    }
}

fn load_gpu_vram_map_from_str(
    toml_src: &str,
) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let vendor_map: VendorVRAMMap = toml::from_str(toml_src)?;
    let mut all = HashMap::new();
    if let Some(amd) = vendor_map.amd {
        all.extend(amd);
    }
    if let Some(nv) = vendor_map.nvidia {
        all.extend(nv);
    }
    Ok(all)
}

#[derive(Debug, Deserialize)]
struct VendorVRAMMap {
    amd: Option<HashMap<String, u64>>,
    nvidia: Option<HashMap<String, u64>>,
}
