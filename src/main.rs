#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod theme;
mod udp;
mod widget;
mod window;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> eframe::Result {
    use udp::UdpClient;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([300.0, 220.0]), //           .with_icon(
        // NOTE: Adding an icon is optional
        //               eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
        //                   .expect("Failed to load icon"),
        //         ),
        ..Default::default()
    };

    let mut udp_client = UdpClient::new();
    udp_client.start();

    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(app::TemplateApp::new(cc, udp_client)))),
    )
}
