use std::process::Command;
use log::info;
use serde::Serialize;
use which::{which_global};
use sysinfo::{System};

#[derive(Serialize, Debug)]
pub struct NodeSoftware {
    docker : String,
    nvidia: String,
    amd: String
}

fn get_version<P: AsRef<std::ffi::OsStr>>(bin: P, args: &[&str]) -> Option<String> {
    let output = Command::new(bin)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&output.stdout));
    if s.trim().is_empty() {
        s.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub fn collect_software_info() -> NodeSoftware {

    info!("Start collecting client software versions");

    let docker_path = which_global("docker").ok();
    let docker_infos = docker_path.as_ref()
        .and_then(|p| get_version(p, &["--version"])).unwrap_or_default();

    info!("docker: {:?}", docker_infos);

    let nvidia_smi_path = which_global("nvidia-smi").ok();
    let nvidia_smi_infos = nvidia_smi_path.as_ref()
        .and_then(|p| get_version(p, &["--version"])).unwrap_or_default();

    info!("nvidia-smi: {:?}", nvidia_smi_infos);

    let amd_smi_path = which_global("amd-smi").ok();
    let amd_smi_infos = amd_smi_path.as_ref()
        .and_then(|p| get_version(p, &["version"])).unwrap_or_default();

    info!("amd-smi: {:?}", amd_smi_infos);

    let software_info = NodeSoftware {
        docker: docker_infos,
        nvidia: nvidia_smi_infos,
        amd: amd_smi_infos
    };

    info!("Finished collecting client software versions");

    software_info
}