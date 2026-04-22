#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod actions;
mod app;
mod theme;
mod udp;
mod widget;
mod window;

pub(crate) fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../images/icon-64.png");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> eframe::Result {
    use udp::UdpClient;

    let app_name = format!("ClicKS Monitor {}", env!("CARGO_PKG_VERSION"));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_min_inner_size([300.0, 220.0])
            .with_title(&app_name)
            .with_icon(load_icon())
            .with_app_id("clicks-monitor"),
        //           .with_icon(
        // NOTE: Adding an icon is optional
        //               eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
        //                   .expect("Failed to load icon"),
        //         ),
        ..Default::default()
    };

    let mut udp_client = UdpClient::new();
    udp_client.start();

    eframe::run_native(
        &app_name,
        native_options,
        Box::new(|cc| Ok(Box::new(app::ClicksMonitorApp::new(cc, udp_client)))),
    )
}
