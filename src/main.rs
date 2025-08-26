mod app;
mod ui;
mod net;
mod reservation;

use app::App;
use net::list_ports;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crossterm::event::{self, Event, KeyCode};
use std::io;
use nix::unistd::Pid;
use nix::sys::signal::{kill, Signal};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Checks if a TCP port is in use on the local machine (cross-platform)
pub fn is_port_in_use(port: u16) -> bool {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;
    if let Ok(sockets) = get_sockets_info(af_flags, proto_flags) {
        for info in sockets {
            if info.local_port() == port {
                return true;
            }
        }
    }
    false
}

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

fn main() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.ports = list_ports();
    app.update_filtered_ports();

    loop {
        terminal.draw(|f| ui::ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if let Some((pid, _)) = app.confirm_kill.clone() {
                    match key.code {
                        KeyCode::Char('y') => {
                            let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
                            app.confirm_kill = None;
                        }
                        KeyCode::Char('n') | KeyCode::Esc => app.confirm_kill = None,
                        _ => {}
                    }
                } else if app.search_mode {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            app.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            app.search.pop();
                            app.update_filtered_ports();
                            app.list_state.select(Some(0));
                        }
                        KeyCode::Char(c) => {
                            app.search.push(c);
                            app.update_filtered_ports();
                            app.list_state.select(Some(0));
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => app.move_down(),
                        KeyCode::Char('k') => app.move_up(),
                        KeyCode::Char('c') => {
                            if let Some(port) = app.current_selection() {
                                if let Some(pid) = port.pid {
                                    let name = port.process.clone().unwrap_or_else(|| "-".into());
                                    app.confirm_kill = Some((pid, name));
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            if let Some(port) = app.current_selection() {
                                if let Some(port_str) = port.local_addr.split(':').last() {
                                    if let Ok(port_num) = port_str.parse::<u16>() {
                                        let service = port.process.clone().unwrap_or_else(|| "unknown".to_string());
                                        app.try_reserve_port(port_num, service);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('u') => {
                            if let Some(port) = app.current_selection() {
                                if let Some(port_str) = port.local_addr.split(':').last() {
                                    if let Ok(port_num) = port_str.parse::<u16>() {
                                        app.try_release_port(port_num);
                                    }
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            app.search_mode = true;
                            app.search.clear();
                            app.update_filtered_ports();
                            app.list_state.select(Some(0));
                        }
                        KeyCode::Char('t') => {
                            app.theme = if app.theme.background == ratatui::style::Color::Black {
                                app.theme.clone()
                            } else {
                                app.theme.clone()
                            };
                            // You can toggle between Theme::default() and Theme::dark() here
                            // For demonstration, just reassign default
                            app.theme = if app.theme.background == ratatui::style::Color::Black {
                                crate::app::Theme::dark()
                            } else {
                                crate::app::Theme::default()
                            };
                        }
                        KeyCode::Char('s') => {
                            app.sort_by = match app.sort_by {
                                crate::app::SortBy::Port => crate::app::SortBy::Process,
                                crate::app::SortBy::Process => crate::app::SortBy::Protocol,
                                crate::app::SortBy::Protocol => crate::app::SortBy::State,
                                crate::app::SortBy::State => crate::app::SortBy::Port,
                            };
                            app.update_filtered_ports();
                        }
                        _ => {}
                    }
                }
            }
        }

        // refresh ports every loop
        app.ports = list_ports();
        app.update_filtered_ports();
    }

    Ok(())
}

