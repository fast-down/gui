use crate::utils::parse_header_hashmap;
use fast_down_ffi::{WriteMethod, fast_down::utils::Proxy};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, path::PathBuf, time::Duration};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub save_dir: PathBuf,
    pub threads: usize,
    pub proxy: Proxy<String>,
    pub headers: HashMap<String, String>,
    pub min_chunk_size: u64,
    pub write_buffer_size: usize,
    pub write_queue_cap: usize,
    pub retry_gap: Duration,
    pub pull_timeout: Duration,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
    pub local_address: Vec<IpAddr>,
    pub max_speculative: usize,
    pub write_method: WriteMethod,
    pub max_concurrency: usize,
    pub retry_times: usize,
    pub chunk_window: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_dir: dirs::download_dir().unwrap_or_default(),
            threads: 32,
            proxy: Proxy::System,
            headers: HashMap::new(),
            min_chunk_size: 8 * 1024 * 1024,
            write_buffer_size: 16 * 1024 * 1024,
            write_queue_cap: 10240,
            retry_gap: Duration::from_millis(500),
            pull_timeout: Duration::from_secs(5),
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
            local_address: Vec::new(),
            max_speculative: 3,
            write_method: WriteMethod::Mmap,
            max_concurrency: 2,
            retry_times: 10,
            chunk_window: 8 * 1024,
        }
    }
}

impl Config {
    pub fn to_ui_config(&self) -> crate::ui::Config {
        crate::ui::Config {
            accept_invalid_certs: self.accept_invalid_certs,
            accept_invalid_hostnames: self.accept_invalid_hostnames,
            headers: self
                .headers
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .join("\n")
                .into(),
            ips: self
                .local_address
                .iter()
                .map(|addr| addr.to_string())
                .join("\n")
                .into(),
            max_speculative: self.max_speculative as i32,
            min_chunk_size: self.min_chunk_size as i32,
            proxy: match self.proxy.as_deref() {
                Proxy::No => "null",
                Proxy::System => "",
                Proxy::Custom(proxy) => proxy,
            }
            .into(),
            pull_timeout_ms: self.pull_timeout.as_millis() as i32,
            retry_gap_ms: self.retry_gap.as_millis() as i32,
            save_dir: self.save_dir.to_string_lossy().as_ref().into(),
            threads: self.threads as i32,
            write_buffer_size: self.write_buffer_size as i32,
            write_method: match self.write_method {
                WriteMethod::Mmap => 0,
                WriteMethod::Std => 1,
            },
            write_queue_cap: self.write_queue_cap as i32,
            max_concurrency: self.max_concurrency as i32,
            retry_times: self.retry_times as i32,
            chunk_window: self.chunk_window as i32,
        }
    }
}

impl From<&crate::ui::Config> for Config {
    fn from(value: &crate::ui::Config) -> Self {
        Self {
            save_dir: value.save_dir.as_str().into(),
            threads: value.threads as usize,
            proxy: match value.proxy.as_str() {
                "" => Proxy::System,
                "null" => Proxy::No,
                proxy => Proxy::Custom(proxy.to_string()),
            },
            headers: parse_header_hashmap(&value.headers),
            min_chunk_size: value.min_chunk_size as u64,
            write_buffer_size: value.write_buffer_size as usize,
            write_queue_cap: value.write_queue_cap as usize,
            retry_gap: Duration::from_millis(value.retry_gap_ms as u64),
            pull_timeout: Duration::from_millis(value.pull_timeout_ms as u64),
            accept_invalid_certs: value.accept_invalid_certs,
            accept_invalid_hostnames: value.accept_invalid_hostnames,
            local_address: value
                .ips
                .as_str()
                .lines()
                .filter_map(|line| line.trim().parse().ok())
                .collect(),
            max_speculative: value.max_speculative as usize,
            write_method: match value.write_method {
                1 => WriteMethod::Std,
                _ => WriteMethod::Mmap,
            },
            max_concurrency: value.max_concurrency as usize,
            retry_times: value.retry_times as usize,
            chunk_window: value.chunk_window as u64,
        }
    }
}
