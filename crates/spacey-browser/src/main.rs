//! Spacey Browser - A minimal web browser using the Spacey JavaScript engine
//!
//! This browser demonstrates integration between:
//! - Spacey JavaScript engine for script execution
//! - Basic HTML rendering
//! - Window management with winit
//! - GPU rendering with wgpu

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod browser;
mod renderer;
mod page;

use browser::Browser;

fn main() {
    env_logger::init();
    
    println!("🚀 Starting Spacey Browser...");
    println!("   JavaScript Engine: Spacey");
    println!("   Rendering: Custom (wgpu + egui)");
    println!();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Spacey Browser")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
            .build(&event_loop)
            .unwrap(),
    );

    let mut browser = Browser::new(window);

    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                browser.resize(size.width, size.height);
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                browser.render();
            }
            Event::AboutToWait => {
                browser.window().request_redraw();
            }
            _ => {}
        }
    });
}
