use eframe::Renderer;

pub mod app;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        renderer: Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 700.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(app::TemplateApp::new(cc)))),
    )
}
