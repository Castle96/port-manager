use procfs::process::{all_processes, FDTarget};
use procfs::net::{tcp, tcp6, udp, udp6};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PortInfo {
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
    pub pid: Option<i32>,
    pub process_name: String,
    pub protocol: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub user: Option<String>,
}

impl PortInfo {
    pub fn matches(&self, query: &str, protocol: Option<&str>, state: Option<&str>, port_range: Option<(u16, u16)>, tags: Option<&[String]>, user: Option<&str>) -> bool {
        let mut result = true;
        if let Some(proto) = protocol {
            result &= self.protocol.eq_ignore_ascii_case(proto);
        }
        if let Some(st) = state {
            result &= self.state.eq_ignore_ascii_case(st);
        }
        if let Some((start, end)) = port_range {
            result &= self.port >= start && self.port <= end;
        }
        if let Some(t) = tags {
            result &= t.iter().all(|tag| self.tags.contains(tag));
        }
        if let Some(u) = user {
            result &= self.user.as_deref() == Some(u);
        }
        result &= self.process_name.to_lowercase().contains(&query.to_lowercase());
        result
    }
}

pub fn list_ports() -> Vec<PortInfo> {
    let mut results = Vec::new();
    let mut tcp_entries = Vec::new();
    if let Ok(tcp4) = tcp() { tcp_entries.extend(tcp4); }
    if let Ok(tcp6) = tcp6() { tcp_entries.extend(tcp6); }
    let mut udp_entries = Vec::new();
    if let Ok(u4) = udp() { udp_entries.extend(u4); }
    if let Ok(u6) = udp6() { udp_entries.extend(u6); }
    let tcp_map: std::collections::HashMap<u64, &procfs::net::TcpNetEntry> =
        tcp_entries.iter().map(|e| (e.inode, e)).collect();
    let udp_map: std::collections::HashMap<u64, &procfs::net::UdpNetEntry> =
        udp_entries.iter().map(|e| (e.inode, e)).collect();
    if let Ok(procs) = all_processes() {
        for prc in procs {
            if let Ok(proc) = prc {
                if let Ok(fds) = proc.fd() {
                    for fd in fds.flatten() {
                        if let FDTarget::Socket(inode) = fd.target {
                            if let Some(entry) = tcp_map.get(&inode) {
                                let local = format!("{}:{}", entry.local_address.ip(), entry.local_address.port());
                                let remote = format!("{}:{}", entry.remote_address.ip(), entry.remote_address.port());
                                results.push(PortInfo {
                                    local_addr: local.clone(),
                                    remote_addr: remote,
                                    state: format!("{:?}", entry.state),
                                    pid: Some(proc.pid()),
                                    process_name: proc.stat().ok().map(|s| s.comm).unwrap_or_default(),
                                    protocol: "TCP".to_string(),
                                    port: entry.local_address.port(),
                                    tags: vec![],
                                    user: None,
                                });
                            }
                            if let Some(entry) = udp_map.get(&inode) {
                                let local = format!("{}:{}", entry.local_address.ip(), entry.local_address.port());
                                let remote = if entry.remote_address.port() == 0 { "-".into() }
                                             else { format!("{}:{}", entry.remote_address.ip(), entry.remote_address.port()) };
                                results.push(PortInfo {
                                    local_addr: local.clone(),
                                    remote_addr: remote,
                                    state: "UDP".into(),
                                    pid: Some(proc.pid()),
                                    process_name: proc.stat().ok().map(|s| s.comm).unwrap_or_default(),
                                    protocol: "UDP".to_string(),
                                    port: entry.local_address.port(),
                                    tags: vec![],
                                    user: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    results
}
// ...existing code...