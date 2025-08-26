use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde_json;
use procfs;

#[derive(Debug, Default)]
pub struct PortReservationManager {
    reservations: Arc<Mutex<HashMap<u16, String>>>, // port -> service name
}

impl PortReservationManager {
    pub fn new() -> Self {
        Self {
            reservations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn is_port_in_use(&self, port: u16) -> bool {
        // Check TCP ports
        if let Ok(tcp) = procfs::net::tcp() {
            if tcp.iter().any(|entry| entry.local_address.port() == port) {
                return true;
            }
        }
        // Check UDP ports
        if let Ok(udp) = procfs::net::udp() {
            if udp.iter().any(|entry| entry.local_address.port() == port) {
                return true;
            }
        }
        false
    }

    pub fn reserve_port(&self, port: u16, service: String) -> Result<(), String> {
        if self.is_port_in_use(port) {
            return Err(format!("Port {} is already in use by another process.", port));
        }
        let mut res = self.reservations.lock().unwrap();
        if res.contains_key(&port) {
            return Err(format!("Port {} is already reserved.", port));
        }
        res.insert(port, service);
        Ok(())
    }

    pub fn release_port(&self, port: u16) -> Result<(), String> {
        let mut res = self.reservations.lock().unwrap();
        if res.remove(&port).is_some() {
            Ok(())
        } else {
            Err(format!("Port {} was not reserved.", port))
        }
    }

    pub fn is_reserved(&self, port: u16) -> bool {
        let res = self.reservations.lock().unwrap();
        res.contains_key(&port)
    }

    pub fn get_all_reservations(&self) -> HashMap<u16, String> {
        self.reservations.lock().unwrap().clone()
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let res = self.reservations.lock().unwrap();
        let serialized = serde_json::to_string(&*res).map_err(|e| e.to_string())?;
        fs::write(path, serialized).map_err(|e| e.to_string())
    }

    pub fn load_from_file(&self, path: &str) -> Result<(), String> {
        if !Path::new(path).exists() {
            return Ok(());
        }
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let deserialized: HashMap<u16, String> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        let mut res = self.reservations.lock().unwrap();
        *res = deserialized;
        Ok(())
    }
}