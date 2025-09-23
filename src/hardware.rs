use log::{error, info};
use serde::Serialize;
use std::fs;
use std::io::Error;
use std::os::unix::io;
use std::path::Path;
use sysinfo::{Disks, System};

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

fn list_pci_gpus() -> Result<Vec<GPU>, Error> {
    let mut all_gpus = Vec::new();

    for entry in fs::read_dir("/sys/bus/pci/devices/")? {
        let pci_entry = entry?;
        let vendor_id = pci_entry.path().join("vendor");
        let device_id = pci_entry.path().join("device");
        let class_code = pci_entry.path().join("class");

        let vendor = fs::read_to_string(&vendor_id).map_err(|e| {
            error!("Failed reading the GPU vendor {e}"); e
        }) ?.trim().to_string();
        let device = fs::read_to_string(&device_id).map_err(|e| {
            error!("Failed reading the GPU device {e}"); e
        }) ?.trim().to_string();
        let class = fs::read_to_string(&class_code).map_err(|e| {
            error!("Failed reading the GPU class {e}"); e
        }) ?.trim().to_string();

        if !class.starts_with("0x03") {
            continue;
        }

        let vendor = lookup_vendor(&vendor);
        let (gpu_name, vram) = lookup_device(&device).unwrap_or(("UNKNOWN", 0));

        if vendor.unwrap_or("UNKNOWN") != "UNKNOWN" {
            all_gpus.push(GPU {
                vendor: vendor.unwrap().to_owned(),
                gpu_type: gpu_name.to_owned(),
                vram: vram as u64,
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

fn lookup_vendor(vendor_id: &str) -> Option<&str> {
    match vendor_id {
        "0x10de" => Some("NVIDIA"),
        "0x1002" => Some("AMD"),
        "0x8086" => Some("Intel"),
        _ => None,
    }
}

fn lookup_device(device_id: &str) -> Option<(&str, u16)> {
    match device_id {
        "0x27b8" => Some(("L4", 24)),
        "0x26b5" => Some(("L40", 48)),
        "0x26b9" => Some(("L40S", 48)),
        "0x20b0" => Some(("A100", 40)),
        "0x20b1" => Some(("A100", 40)),
        "0x20b2" => Some(("A100", 80)),
        "0x20b3" => Some(("A100", 80)),
        "0x740f" => Some(("MI210", 64)),
        _ => None,
    }
}