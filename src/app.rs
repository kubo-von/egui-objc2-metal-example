use core::{cell::OnceCell, ptr::NonNull};
use objc2::rc::Retained;
use objc2::runtime::{Object, ProtocolObject};
use objc2_foundation::{ns_string, MainThreadMarker};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice,
    MTLLibrary, MTLOrigin, MTLPixelFormat, MTLRegion, MTLRenderCommandEncoder,
    MTLRenderPassColorAttachmentDescriptor, MTLRenderPassDescriptor, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState, MTLSize, MTLStoreAction, MTLTexture, MTLTextureDescriptor,
    MTLTextureUsage,
};
use std::sync::Arc;

#[derive(Copy, Clone)]
#[repr(C)]
struct SceneProperties {
    offset_x: f32,
    offset_y: f32,
}

pub struct TemplateApp {
    // Example stuff:
    label: String,
    value: f32,
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    command_queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    pipeline_state: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    texture_id: Option<egui::TextureId>,
    pan: egui::Pos2,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        unsafe {
            let device = MTLCreateSystemDefaultDevice().expect("Failed to create Metal device");
            let command_queue = device
                .newCommandQueue()
                .expect("Failed to create command queue");

            // Load shader library and compile the pipeline
            let library = device
                .newLibraryWithSource_options_error(
                    ns_string!(include_str!("triangle.metal")),
                    None,
                )
                .expect("Failed to create library");

            let vertex_function = library.newFunctionWithName(ns_string!("vertex_main"));
            let fragment_function = library.newFunctionWithName(ns_string!("fragment_main"));

            let pipeline_descriptor = MTLRenderPipelineDescriptor::new();
            pipeline_descriptor.setVertexFunction(vertex_function.as_deref());
            pipeline_descriptor.setFragmentFunction(fragment_function.as_deref());
            pipeline_descriptor
                .colorAttachments()
                .objectAtIndexedSubscript(0)
                .setPixelFormat(MTLPixelFormat::BGRA8Unorm); // MTLPixelFormatBGRA8Unorm

            let pipeline_state = device
                .newRenderPipelineStateWithDescriptor_error(&pipeline_descriptor)
                .expect("Failed to create pipeline state");

            // Create a Metal texture
            let texture_desc = MTLTextureDescriptor::new();
            texture_desc.setPixelFormat(MTLPixelFormat::BGRA8Unorm); // MTLPixelFormatBGRA8Unorm
            texture_desc.setWidth(256);
            texture_desc.setHeight(256);
            texture_desc.setUsage(MTLTextureUsage::ShaderRead | MTLTextureUsage::RenderTarget);
            let texture = device
                .newTextureWithDescriptor(&texture_desc)
                .expect("newTextureWithDescriptor() failed");

            let texture_id = texture.gpuResourceID().to_raw();

            Self {
                // Example stuff:
                label: "Hello World!".to_owned(),
                value: 2.7,
                device,
                command_queue,
                pipeline_state,
                texture,
                texture_id: Some(egui::TextureId::User(texture_id)),
                pan: egui::Pos2 { x: 0.0, y: 0.0 },
            }
        }
    }

    fn render_triangle(&self) {
        // Create a command buffer
        let command_buffer = self
            .command_queue
            .commandBuffer()
            .expect("Failed to create command buffer");

        // Create a render pass descriptor
        let render_pass_desc = unsafe { MTLRenderPassDescriptor::new() };

        let color_attachment_desc = MTLRenderPassColorAttachmentDescriptor::new();
        color_attachment_desc.setLoadAction(objc2_metal::MTLLoadAction::Clear);
        color_attachment_desc.setStoreAction(MTLStoreAction::Store);
        color_attachment_desc.setTexture(Some(&self.texture));

        let mut color_attachments = render_pass_desc.colorAttachments();
        unsafe {
            color_attachments.setObject_atIndexedSubscript(Some(&color_attachment_desc), 0);
        };

        // Create a render command encoder
        let encoder = command_buffer
            .renderCommandEncoderWithDescriptor(&render_pass_desc)
            .expect("failed to create render command encoder");

        // compute the scene properties
        let scene_properties_data = &SceneProperties {
            offset_x: self.pan.x as f32,
            offset_y: self.pan.y as f32,
        };
        // write the scene properties to the vertex shader argument buffer at index 0
        let scene_properties_bytes = NonNull::from(scene_properties_data);

        // Set pipeline state and draw
        encoder.setRenderPipelineState(&self.pipeline_state);
        unsafe {
            encoder.setVertexBytes_length_atIndex(
                scene_properties_bytes.cast::<core::ffi::c_void>(),
                core::mem::size_of_val(scene_properties_data),
                0,
            );
            encoder.setFragmentBytes_length_atIndex(
                scene_properties_bytes.cast::<core::ffi::c_void>(),
                core::mem::size_of_val(scene_properties_data),
                0,
            );
        };
        unsafe {
            encoder.drawPrimitives_vertexStart_vertexCount(
                objc2_metal::MTLPrimitiveType::Triangle,
                0,
                6,
            );
        }

        encoder.endEncoding();

        // Commit the command buffer
        command_buffer.commit();
        unsafe {
            command_buffer.waitUntilCompleted();
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        self.render_triangle();

        let width = 256;
        let height = 256;
        // Assuming RGBA8Unorm format (4 bytes per pixel)
        let bytes_per_pixel = 4;
        let bytes_per_row = width * bytes_per_pixel;

        // Calculate total data size
        let data_size = (bytes_per_row * height) as usize;

        // Allocate a buffer for the texture data
        let mut data = vec![0u8; data_size];

        unsafe {
            let region = MTLRegion {
                origin: MTLOrigin { x: 0, y: 0, z: 0 },
                size: MTLSize {
                    width: 256,
                    height: 256,
                    depth: 1,
                },
            };
            self.texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                NonNull::new(data.as_mut_ptr() as *mut std::ffi::c_void)
                    .expect("could not create nonNull"),
                bytes_per_row,
                region,
                0,
            );
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe MTL texture example - press middle mouse and drag");

            let do_pan = ctx.input(|i| i.pointer.middle_down()); // && ui.rect_contains_pointer(clip);
            if do_pan {
                self.pan += ctx.input(|i| i.pointer.delta());
            };

            let image = egui::ColorImage::from_rgba_unmultiplied([256, 256], &data);
            let texture = ctx.load_texture("metal texture", image, Default::default());
            ui.image(egui::load::SizedTexture::new(
                texture.id(),
                ui.available_size(),
            ));

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
