use std::sync::Arc;

use smol::block_on;
use wgpu::{Device, Instance, InstanceDescriptor, Queue, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{self, Window},
};

struct State {
    window: Arc<Window>,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let instance = Instance::new(&InstanceDescriptor {
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to create device");

        let size = window.inner_size();

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
        };

        state.configure_surface();

        state
    }

    fn configure_surface(&self) {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &config);
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface with the new size
        self.configure_surface();
    }

    fn render(&mut self) {
        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    fn get_window(&self) -> &Window {
        &self.window
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create window"),
        );

        let state = block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: event::WindowEvent,
    ) {
        let state = self.state.as_mut().expect("State is not initialized");
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                println!("Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Failed to run app");
}
