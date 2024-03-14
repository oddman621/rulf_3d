//use std::rc::Rc;
use std::sync::Arc;

use winit::window::Window;

//렌더링엔진은 크게 3파트로 나눔: GUI, 미니맵, 1인칭
pub mod minimap;

pub struct WebGPU
{
	pub surface: wgpu::Surface<'static>,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration
}


impl WebGPU {
	pub fn reconfigure_surface_size(&mut self, width: u32, height: u32) {
		let max_texture_extent = self.device.limits().max_texture_dimension_2d;
		self.config.width = std::cmp::min(width, max_texture_extent);
		self.config.height = std::cmp::min(height, max_texture_extent);
		self.surface.configure(&self.device, &self.config);
	}
	
	pub fn new(window: Arc<Window>) -> Self {
		let window_size = window.inner_size();
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
		let surface = instance.create_surface(window).expect("Failed to create surface.");
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

		Self { surface, device, queue, config }
	}
}