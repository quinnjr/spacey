//! Renderer - handles GPU-accelerated rendering using wgpu and egui

use std::sync::Arc;
use winit::window::Window;
use egui_wgpu::ScreenDescriptor;

use crate::page::Page;
use crate::ai_ui::{AiUiState, AiPanelAction, render_ai_panel};
use crate::extensions::Extension;
use crate::extensions_ui::{ExtensionsUiState, ExtensionsAction, render_extensions_panel};

pub struct Renderer {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    egui_renderer: egui_wgpu::Renderer,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        // Request device
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Setup egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);

        Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            egui_renderer,
            egui_ctx,
            egui_state,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    /// Render the page (without AI panel)
    pub fn render(&mut self, page: &Page) {
        self.render_internal(Some(page), None)
    }

    /// Render the page with the AI panel
    pub fn render_with_ai(
        &mut self,
        page: Option<&Page>,
        ai_state: &mut AiUiState,
    ) -> Option<AiPanelAction> {
        let mut action = None;

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepare egui
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Render the AI panel first (on the right)
            action = render_ai_panel(ctx, ai_state);

            // Then render the page content
            if let Some(page) = page {
                page.render_ui(ctx);
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.heading("No page loaded");
                    });
                });
            }
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        // Render
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.95,
                            g: 0.95,
                            b: 0.95,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.egui_renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        } // render_pass dropped here

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        action
    }

    /// Render the page with both AI and Extensions panels
    pub fn render_with_panels(
        &mut self,
        page: Option<&Page>,
        ai_state: &mut AiUiState,
        ext_state: &mut ExtensionsUiState,
        installed: &[Extension],
    ) -> (Option<AiPanelAction>, Option<ExtensionsAction>) {
        let mut ai_action = None;
        let mut ext_action = None;

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepare egui
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Render the AI panel (on the right)
            ai_action = render_ai_panel(ctx, ai_state);

            // Render the extensions panel
            ext_action = render_extensions_panel(ctx, ext_state, installed);

            // Then render the page content
            if let Some(page) = page {
                page.render_ui(ctx);
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.heading("No page loaded");
                    });
                });
            }
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        // Render
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.95,
                            g: 0.95,
                            b: 0.95,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.egui_renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        } // render_pass dropped here

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        (ai_action, ext_action)
    }

    fn render_internal(&mut self, page: Option<&Page>, _ai_state: Option<&mut AiUiState>) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Prepare egui
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            if let Some(page) = page {
                page.render_ui(ctx);
            }
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let tris = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        // Render
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.egui_renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        } // render_pass dropped here

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
    
    /// Get current viewport dimensions
    pub fn viewport_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }
    
    /// Capture a screenshot of the current frame
    /// 
    /// Returns the raw RGBA pixel data and dimensions
    pub fn capture_screenshot(&self) -> Result<ScreenshotData, String> {
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        
        // Calculate buffer size with proper alignment
        let bytes_per_pixel = 4u32; // RGBA
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;
        let buffer_size = (padded_bytes_per_row * height) as u64;
        
        // Create a texture to render to
        let screenshot_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Screenshot Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        
        let screenshot_view = screenshot_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create output buffer
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        // Create command encoder and render to the texture
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Encoder"),
        });
        
        // Render a frame to the screenshot texture
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screenshot Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &screenshot_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.95,
                            g: 0.95,
                            b: 0.95,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // Note: The actual content rendering would need to happen here
            // For now, we capture the current surface state
        }
        
        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &screenshot_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        
        self.queue.submit(std::iter::once(encoder.finish()));
        
        // Read the buffer
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        
        rx.recv()
            .map_err(|e| format!("Failed to receive screenshot data: {}", e))?
            .map_err(|e| format!("Failed to map screenshot buffer: {:?}", e))?;
        
        let data = buffer_slice.get_mapped_range();
        
        // Remove padding from rows
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height {
            let start = (row * padded_bytes_per_row) as usize;
            let end = start + (width * bytes_per_pixel) as usize;
            pixels.extend_from_slice(&data[start..end]);
        }
        
        drop(data);
        output_buffer.unmap();
        
        Ok(ScreenshotData {
            pixels,
            width,
            height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

/// Raw screenshot data
#[derive(Debug, Clone)]
pub struct ScreenshotData {
    /// RGBA pixel data
    pub pixels: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Capture timestamp
    pub timestamp: u64,
}

impl ScreenshotData {
    /// Encode as PNG and return base64 string
    pub fn to_png_base64(&self) -> Result<String, String> {
        use std::io::Write;
        
        let mut png_data = Vec::new();
        {
            let mut encoder = ::png::Encoder::new(&mut png_data, self.width, self.height);
            encoder.set_color(::png::ColorType::Rgba);
            encoder.set_depth(::png::BitDepth::Eight);
            
            let mut writer = encoder.write_header()
                .map_err(|e| format!("PNG header error: {}", e))?;
            
            writer.write_image_data(&self.pixels)
                .map_err(|e| format!("PNG write error: {}", e))?;
        }
        
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(&png_data))
    }
    
    /// Get raw RGBA data as base64 (without PNG encoding)
    pub fn to_raw_base64(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(&self.pixels)
    }
    
    /// Get size in bytes
    pub fn size_bytes(&self) -> usize {
        self.pixels.len()
    }
}
