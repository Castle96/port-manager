#[cfg(target_os = "linux")]
use procfs::process::{all_processes, FDTarget};
#[cfg(target_os = "linux")]
use procfs::net::{tcp, tcp6, udp, udp6};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

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
    #[cfg(target_os = "linux")]
    {
        let mut results = Vec::new();
        // ...existing Linux code...
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
                                        local_addr: local,
                                        remote_addr: remote,
                                        state: format!("{:?}", entry.state),
                                        pid: Some(proc.pid()),
                                        process: proc.stat().ok().map(|s| s.comm),
                                    });
                                }
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
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let mut results = Vec::new();
        let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        if let Ok(sockets) = get_sockets_info(af_flags, proto_flags) {
            for info in sockets {
                match info.protocol_socket_info {
                    ProtocolSocketInfo::Tcp(tcp) => {
                        results.push(PortInfo {
                            local_addr: format!("{}:{}", tcp.local_addr, tcp.local_port),
                            remote_addr: format!("{}:{}", tcp.remote_addr, tcp.remote_port),
                            state: format!("{:?}", tcp.state),
                            pid: info.associated_pids.get(0).cloned(),
                            process: None, // Not available cross-platform
                        });
                    }
                    ProtocolSocketInfo::Udp(udp) => {
                        results.push(PortInfo {
                            local_addr: format!("{}:{}", udp.local_addr, udp.local_port),
                            remote_addr: format!("{}:{}", udp.remote_addr, udp.remote_port),
                            state: "UDP".to_string(),
                            pid: info.associated_pids.get(0).cloned(),
                            process: None,
                        });
                    }
                }
            }
        }
        results
    }
}

