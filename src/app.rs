use crate::reservation::PortReservationManager;
use ratatui::style::Color;
#[derive(Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub header_fg: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            highlight_bg: Color::Blue,
            highlight_fg: Color::Yellow,
            header_fg: Color::Cyan,
        }
    }

    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::Gray,
            highlight_bg: Color::DarkGray,
            highlight_fg: Color::LightYellow,
            header_fg: Color::LightCyan,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SortBy {
    Port,
    Process,
    Protocol,
    State,
}

use ratatui::widgets::TableState;
use crate::net::PortInfo;

pub struct App {
    pub search: String,
    pub search_mode: bool,
    pub ports: Vec<PortInfo>,
    pub filtered_ports: Vec<PortInfo>,
    pub list_state: TableState,        // track selected row
    pub confirm_kill: Option<(i32, String)>,
    pub theme: Theme,
    pub sort_by: SortBy,
    pub reservation_manager: PortReservationManager,
    pub reservation_popup: Option<(u16, String)>, // (port, service)
    pub reservation_error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            search_mode: false,
            ports: Vec::new(),
            filtered_ports: Vec::new(),
            list_state: TableState::default(),
            confirm_kill: None,
            theme: Theme::default(),
            sort_by: SortBy::Port,
            reservation_manager: PortReservationManager::new(),
            reservation_popup: None,
            reservation_error: None,
        }
    }

    pub fn update_filtered_ports(&mut self) {
        if self.search.is_empty() {
            self.filtered_ports = self.ports.clone();
        } else {
            self.filtered_ports = self.ports
                .iter()
                .cloned()
                .filter(|p| p.matches(&self.search))
                .collect();
        }
        self.sort_ports();
    }

    pub fn sort_ports(&mut self) {
        match self.sort_by {
            SortBy::Port => self.filtered_ports.sort_by_key(|p| p.local_addr.clone()),
            SortBy::Process => self.filtered_ports.sort_by_key(|p| p.process.clone().unwrap_or_default()),
            SortBy::Protocol => self.filtered_ports.sort_by_key(|p| p.state.clone()),
            SortBy::State => self.filtered_ports.sort_by_key(|p| p.state.clone()),
        }
    }

    pub fn selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn move_down(&mut self) {
        let len = self.filtered_ports.len();
        if len == 0 { return; }

        let i = self.selected_index();
        let next = if i >= len - 1 { 0 } else { i + 1 };
        self.list_state.select(Some(next));
    }

    pub fn move_up(&mut self) {
        let len = self.filtered_ports.len();
        if len == 0 { return; }

        let i = self.selected_index();
        let prev = if i == 0 { len - 1 } else { i - 1 };
        self.list_state.select(Some(prev));
    }

    pub fn current_selection(&self) -> Option<&PortInfo> {
        self.filtered_ports.get(self.selected_index())
    }

    // Helper to reserve port from UI
    pub fn try_reserve_port(&mut self, port: u16, service: String) {
        match self.reservation_manager.reserve_port(port, service.clone()) {
            Ok(_) => self.reservation_popup = Some((port, service)),
            Err(e) => self.reservation_error = Some(e),
        }
    }

    pub fn try_release_port(&mut self, port: u16) {
        match self.reservation_manager.release_port(port) {
            Ok(_) => self.reservation_popup = Some((port, "released".to_string())),
            Err(e) => self.reservation_error = Some(e),
        }
    }
}

