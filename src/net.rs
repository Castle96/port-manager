use procfs::process::{all_processes, FDTarget};
use procfs::net::{tcp, tcp6, udp, udp6};

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
    pub pid: Option<i32>,
    pub process: Option<String>,
}

impl PortInfo {
    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.local_addr.to_lowercase().contains(&query)
            || self.remote_addr.to_lowercase().contains(&query)
            || self.state.to_lowercase().contains(&query)
            || self.process.as_ref().map(|p| p.to_lowercase().contains(&query)).unwrap_or(false)
    }
}

pub fn list_ports() -> Vec<PortInfo> {
    let mut results = Vec::new();

    // Collect TCP entries
    let mut tcp_entries = Vec::new();
    if let Ok(tcp4) = tcp() { tcp_entries.extend(tcp4); }
    if let Ok(tcp6) = tcp6() { tcp_entries.extend(tcp6); }

    // Collect UDP entries
    let mut udp_entries = Vec::new();
    if let Ok(u4) = udp() { udp_entries.extend(u4); }
    if let Ok(u6) = udp6() { udp_entries.extend(u6); }

    // Map inodes to TCP sockets
    let tcp_map: std::collections::HashMap<u64, &procfs::net::TcpNetEntry> =
        tcp_entries.iter().map(|e| (e.inode, e)).collect();

    // Map inodes to UDP sockets
    let udp_map: std::collections::HashMap<u64, &procfs::net::UdpNetEntry> =
        udp_entries.iter().map(|e| (e.inode, e)).collect();

    if let Ok(procs) = all_processes() {
        for prc in procs {
            if let Ok(proc) = prc {
                if let Ok(fds) = proc.fd() {
                    for fd in fds.flatten() {
                        if let FDTarget::Socket(inode) = fd.target {
                            // Check TCP first
                            if let Some(entry) = tcp_map.get(&inode) {
                                let local = format!("{}:{}", entry.local_address.ip(), entry.local_address.port());
                                let remote = format!("{}:{}", entry.remote_address.ip(), entry.remote_address.port());
                                results.push(PortInfo {
                                    local_addr: local,
                                    remote_addr: remote,
                                    state: format!("{:?}", entry.state),
                                    pid: Some(proc.pid()),
                                    process: proc.stat().ok().map(|s| s.comm),
                                });
                            }
                            // Check UDP next
                            if let Some(entry) = udp_map.get(&inode) {
                                let local = format!("{}:{}", entry.local_address.ip(), entry.local_address.port());
                                let remote = if entry.remote_address.port() == 0 { "-".into() }
                                             else { format!("{}:{}", entry.remote_address.ip(), entry.remote_address.port()) };
                                results.push(PortInfo {
                                    local_addr: local,
                                    remote_addr: remote,
                                    state: "UDP".into(),
                                    pid: Some(proc.pid()),
                                    process: proc.stat().ok().map(|s| s.comm),
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

