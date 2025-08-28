use eframe::egui;
use tray_icon::{TrayIconBuilder, Icon};
use tray_icon::menu::{Menu, MenuItem};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, SocketInfo};

fn get_ports() -> Vec<SocketInfo> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    get_sockets_info(af_flags, proto_flags).unwrap_or_default()
}

pub struct PortManagerApp {
    filter: String,
}

impl Default for PortManagerApp {
    fn default() -> Self {
        Self {
            filter: String::new(),
        }
    }
}

impl eframe::App for PortManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Port Manager");
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter);
            });
            ui.separator();
            let ports = get_ports();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for socket in ports.iter().filter(|s| {
                    self.filter.is_empty() || format!("{:?}", s).contains(&self.filter)
                }) {
                    ui.label(format!("{:?}", socket));
                }
            });
        });
    }
}

fn main() {
    // Initialize GTK before using tray-icon
    gtk::init().expect("Failed to initialize GTK");
    let native_options = eframe::NativeOptions::default();
    // Load icon PNG file as buffer
    // Create a 32x32 red RGBA icon as a placeholder
    let width = 32;
    let height = 32;
    let mut rgba = vec![0u8; width * height * 4];
    for i in 0..(width * height) {
        rgba[i * 4] = 255; // R
        rgba[i * 4 + 1] = 0;   // G
        rgba[i * 4 + 2] = 0;   // B
        rgba[i * 4 + 3] = 255; // A
    }
    let icon = Icon::from_rgba(rgba, width as u32, height as u32).expect("Failed to create icon");

    // Create tray menu and items
    let mut menu = Menu::new();
    let show_item = MenuItem::new("Show", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append_items(&[&show_item, &quit_item]);

    // Build tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
        .expect("Failed to create tray icon");

    let _ = eframe::run_native(
        "Port Manager",
        native_options,
        Box::new(|_cc| Box::new(PortManagerApp::default())),
    );
}
