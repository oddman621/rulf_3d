use std::f32::consts::PI;
use crate::{webgpu::WebGPU, game::GameWorld};

mod wall_raycast;
mod wall;
mod floorceil;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SurfaceInfo {
	width: u32,
	height: u32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RaycastData {
	distance: f32,
	texid: u32,
	u_offset: f32
}

pub struct Renderer {
	wall_render: wall::WallRender,
	depth_texture: wgpu::Texture
}

impl Renderer {
	pub fn render(&mut self, webgpu: &WebGPU, game_world: &GameWorld, clear_color: &wgpu::Color) {
		let output = webgpu.surface.get_current_texture().unwrap();
		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		if self.depth_texture.size() != size {
			self.depth_texture.destroy();
			self.depth_texture = webgpu.device.create_texture(&wgpu::TextureDescriptor {
				label: Some("Renderer::depth_texture from render()"),
				mip_level_count: 1,
				sample_count: 1,
				size,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Depth32Float,
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
				view_formats: &[]
			});
		}
		let depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
		
		let surface_info = SurfaceInfo {
			width: output.texture.width(),
			height: output.texture.height()
		};
		let raycount = 300;
		if let Ok(raycast) = wall_raycast::multiple_raycast(
			&game_world.get_walls(), game_world.get_grid_size(), 
			game_world.get_player_position(), game_world.get_player_forward_vector(), 
			PI/2.0, raycount, 50
		) {
			let raycast_data: Vec<RaycastData> = raycast.into_iter().map(|(d, t, u)| RaycastData {
				distance: d, texid: t, u_offset: u
			}).collect();
			webgpu.queue.write_buffer(&self.wall_render.surface_info_buffer, 0, bytemuck::cast_slice(&[surface_info]));
			webgpu.queue.write_buffer(&self.wall_render.raycast_data_array_buffer, 0, bytemuck::cast_slice(&[raycast_data.len() as u32]));
			webgpu.queue.write_buffer(&self.wall_render.raycast_data_array_buffer, std::mem::size_of::<u32>() as u64, bytemuck::cast_slice(&raycast_data));

			let mut encoder = webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("firstperson::Renderer::render() clearcolor render pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(*clear_color),
						store: wgpu::StoreOp::Store
					}
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
					view: &depth_view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: wgpu::StoreOp::Store
					}),
					stencil_ops: None
				}),
				..Default::default()
			});

			render_pass.set_pipeline(&self.wall_render.pipeline);
			render_pass.set_bind_group(0, &self.wall_render.bind_groups[0], &[]);
			render_pass.set_bind_group(1, &self.wall_render.bind_groups[1], &[]);
			render_pass.draw(0..4, 0..1);

			drop(render_pass);
			webgpu.queue.submit(Some(encoder.finish()));
			output.present();
		}
	}
	pub fn new(webgpu: &WebGPU) -> Self {
		let depth_texture = webgpu.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Renderer::depth_texture"),
			mip_level_count: 1,
			sample_count: 1,
			size: wgpu::Extent3d {
				width: 1, height: 1, depth_or_array_layers: 1
			},
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Depth32Float,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[]
		});
		Self {
			wall_render: wall::WallRender::new(webgpu), depth_texture
		}
	}
}

