use std::path::Path;
use std::process::Command;
use sysinfo::{Disks, System};

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("Total memory: {} GiB", bytes_to_gib(sys.total_memory()));
    println!("Total number of CPU threads: {}", sys.cpus().len());

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
                    println!("--- Vendor = {}, Model = {} with {} GB of VRAM", v, gpu_name, vram);
                }
            }
        }
    }
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
