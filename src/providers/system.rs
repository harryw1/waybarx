use serde::{Deserialize, Serialize};
use sysinfo::{System, Disks, Networks};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub usage_percent: f32,
    pub cores: Vec<CoreInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub id: usize,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: Vec<NetworkInfo>,
}

pub struct SystemProvider {
    sys: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    networks: Arc<Mutex<Networks>>,
}

impl SystemProvider {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys: Arc::new(Mutex::new(sys)),
            disks: Arc::new(Mutex::new(Disks::new_with_refreshed_list())),
            networks: Arc::new(Mutex::new(Networks::new_with_refreshed_list())),
        }
    }

    pub fn refresh(&self) {
        if let Ok(mut sys) = self.sys.lock() {
            sys.refresh_cpu_all();
            sys.refresh_memory();
        }
        if let Ok(mut disks) = self.disks.lock() {
            disks.refresh_list();
            disks.refresh();
        }
        if let Ok(mut networks) = self.networks.lock() {
            networks.refresh();
        }
    }

    pub fn get_info(&self) -> Option<SystemInfo> {
        let cpu = self.get_cpu_info()?;
        let memory = self.get_memory_info()?;
        let disks = self.get_disk_info();
        let network = self.get_network_info();

        Some(SystemInfo {
            cpu,
            memory,
            disks,
            network,
        })
    }

    fn get_cpu_info(&self) -> Option<CpuInfo> {
        let sys = self.sys.lock().ok()?;

        let cores: Vec<CoreInfo> = sys.cpus().iter().enumerate().map(|(id, cpu)| {
            CoreInfo {
                id,
                usage_percent: cpu.cpu_usage(),
            }
        }).collect();

        let usage_percent = if !cores.is_empty() {
            cores.iter().map(|c| c.usage_percent).sum::<f32>() / cores.len() as f32
        } else {
            0.0
        };

        Some(CpuInfo {
            usage_percent,
            cores,
        })
    }

    fn get_memory_info(&self) -> Option<MemoryInfo> {
        let sys = self.sys.lock().ok()?;

        let total_bytes = sys.total_memory();
        let used_bytes = sys.used_memory();
        let available_bytes = sys.available_memory();
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f64 / total_bytes as f64 * 100.0) as f32
        } else {
            0.0
        };

        Some(MemoryInfo {
            total_bytes,
            used_bytes,
            available_bytes,
            usage_percent,
        })
    }

    fn get_disk_info(&self) -> Vec<DiskInfo> {
        let Ok(disks) = self.disks.lock() else {
            return Vec::new();
        };

        disks.iter().map(|disk| {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);
            let usage_percent = if total_bytes > 0 {
                (used_bytes as f64 / total_bytes as f64 * 100.0) as f32
            } else {
                0.0
            };

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_bytes,
                used_bytes,
                available_bytes,
                usage_percent,
            }
        }).collect()
    }

    fn get_network_info(&self) -> Vec<NetworkInfo> {
        let Ok(networks) = self.networks.lock() else {
            return Vec::new();
        };

        networks.iter().map(|(interface, data)| {
            NetworkInfo {
                interface: interface.clone(),
                received_bytes: data.received(),
                transmitted_bytes: data.transmitted(),
            }
        }).collect()
    }
}

impl Default for SystemProvider {
    fn default() -> Self {
        Self::new()
    }
}
