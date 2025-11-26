use log::{error, info};
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize, Debug)]
pub struct NodeSystem {
    os: String,
    kernel: String,
}

pub fn collect_system_info() -> NodeSystem {
    info!("Start collecting system information");

    let os = System::long_os_version()
        .map(|os| {
            info!("Operating system: {:?}", os);
            os
        })
        .unwrap_or_else(|| {
            error!("Unknown operating system found");
            String::new()
        });

    let kernel = System::kernel_long_version();
    info!("Kernel Version: {}", kernel);
    let system: NodeSystem = NodeSystem { os, kernel };

    info!("Finished collecting system information");
    system
}
