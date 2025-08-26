use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::is_port_in_use;

/// Struct to manage port reservations
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

    /// Reserve a port for a service
    pub fn reserve_port(&self, port: u16, service: String) -> Result<(), String> {
        let mut res = self.reservations.lock().unwrap();
        if res.contains_key(&port) {
            return Err(format!("Port {} is already reserved.", port));
        }
        if is_port_in_use(port) {
            return Err(format!("Port {} is currently in use by another process.", port));
        }
        res.insert(port, service);
        Ok(())
    }

    /// Release a reserved port
    pub fn release_port(&self, port: u16) -> Result<(), String> {
        let mut res = self.reservations.lock().unwrap();
        if res.remove(&port).is_some() {
            Ok(())
        } else {
            Err(format!("Port {} was not reserved.", port))
        }
    }

    /// Check if a port is reserved
    pub fn is_reserved(&self, port: u16) -> bool {
        let res = self.reservations.lock().unwrap();
        res.contains_key(&port)
    }

    /// Get the service name for a reserved port
    pub fn get_service(&self, port: u16) -> Option<String> {
        let res = self.reservations.lock().unwrap();
        res.get(&port).cloned()
    }
}
