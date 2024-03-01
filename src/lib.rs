use std::{sync::Arc, time::{Duration, Instant}};
use winit::{
    dpi::LogicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub fn run_dev(t: impl GameLoop)
{
	let engine = Engine::init();
    let _ = engine.game_loop(t);
}

pub trait GameLoop
{
    fn startup(&mut self, device: &wgpu::Device, surface_format: wgpu::TextureFormat);
    fn process(&mut self, delta: f64);
    fn render(&mut self, device: &wgpu::Device, surface: &wgpu::Surface, queue: &wgpu::Queue);
}

struct Engine
{
	event_loop: EventLoop<()>,
	window: Arc<Window>,
	_instance: wgpu::Instance,
	surface: wgpu::Surface<'static>,
	_adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration
}

impl Engine
{
    // Window Settings
    const WINDOW_WIDTH: u32 = 800;
    const WINDOW_HEIGHT: u32 = 600;
    const WINDOW_TITLE: &'static str = "Rulf 3D";

	fn init() -> Self
	{
		let event_loop = EventLoop::new().expect("Failed to create EventLoop.");
		let window_size = LogicalSize::new(Engine::WINDOW_WIDTH, Engine::WINDOW_HEIGHT);
		let window = Arc::new(
			WindowBuilder::new()
				.with_title(Engine::WINDOW_TITLE)
				.with_inner_size(window_size)
				.build(&event_loop).expect("Failed to build winit window.")
		);

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
		let surface = instance.create_surface(window.clone()).expect("Failed to create surface.");
		let adapter = pollster::block_on(instance.request_adapter(
			&wgpu::RequestAdapterOptions
			{
				compatible_surface: Some(&surface), 
				..Default::default()
			}
		)).expect("Failed to request adapter.");
		let (device, queue) = pollster::block_on(adapter.request_device(
			&wgpu::DeviceDescriptor
			{
				required_features: wgpu::Features::empty(),
				required_limits: wgpu::Limits::default(),
				label: None
			}, 
			None
		)).expect("Failed to request device from adapter.");

		let surface_format = surface.get_capabilities(&adapter).formats
			.into_iter().find(|f|f == &wgpu::TextureFormat::Rgba8UnormSrgb)
			.expect("Surface is not support RgbaUnormSrgb.");
		let config = wgpu::SurfaceConfiguration
		{
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: window_size.width,
			height: window_size.height,
			present_mode: wgpu::PresentMode::AutoVsync,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![],
			desired_maximum_frame_latency: 2
		};
		surface.configure(&device, &config);

		Self { event_loop, window, _instance: instance, surface, _adapter: adapter, device, queue, config }
	}

    fn game_loop(
        mut self,
        mut t: impl GameLoop
    ) -> Result<(), winit::error::EventLoopError>
    {
        let process_tickrate = Duration::from_secs_f64(60.0f64.recip());
        let mut last_process_tick = Instant::now();
        self.event_loop.run(
            move |event, elwt| 
            match event 
            {
                Event::WindowEvent { event, window_id } if window_id == self.window.id() => 
                match event 
                {
                    WindowEvent::RedrawRequested =>
                    {
                        // NOTE: draw things
                        t.render(&self.device, &self.surface, &self.queue);
                    },
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) if physical_size.width > 0 && physical_size.height > 0 => 
                    {
                        let max_texture_extent = self.device.limits().max_texture_dimension_2d;
                        self.config.width = std::cmp::min(physical_size.width, max_texture_extent);
                        self.config.height = std::cmp::min(physical_size.height, max_texture_extent);
                        self.surface.configure(&self.device, &self.config);
                    },
                    _ => ()
                },
                Event::NewEvents(StartCause::Init) =>
                {
                    // NOTE: startup things
                    t.startup(&self.device, self.config.format);
                }
                Event::NewEvents(StartCause::Poll | StartCause::ResumeTimeReached { .. } | StartCause::WaitCancelled { .. }) =>
                {
                    let last_process_time = Instant::now().duration_since(last_process_tick);
                    if last_process_time >= process_tickrate
                    {
                        let delta = last_process_time.as_secs_f64();
                        last_process_tick = Instant::now(); // doing process point
                        
                        // NOTE: process things
                        t.process(delta);

                        self.window.request_redraw();
                    }
                },
                Event::AboutToWait =>
                {
                    let last_process_time = Instant::now().duration_since(last_process_tick);
                    if last_process_time >= process_tickrate
                    {
                        elwt.set_control_flow(ControlFlow::Poll);
                    }
                    else
                    {
                        elwt.set_control_flow(ControlFlow::WaitUntil(Instant::now() + process_tickrate.mul_f64(0.6) - last_process_time)); 
                    }
                },
                _ => ()
            }
        )
    }
}