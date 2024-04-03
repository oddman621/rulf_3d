use std::f32::consts::PI;
use crate::{game::{GameWorld, TileType}, webgpu::WebGPU};

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

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraInfo {
	pos: glam::Vec2,
	pos_z: f32,
	len: f32,
	leftmost_ray: glam::Vec2,
	rightmost_ray: glam::Vec2
}

pub struct Renderer {
	pub fov: f32,
	pub raycount: u32,
	floorceil_data: floorceil::Data,
	wall_data: wall::Data,
	depth_texture: wgpu::Texture
}

impl Renderer {
	pub fn render(&mut self, webgpu: &WebGPU, game_world: &GameWorld, clear_color: &wgpu::Color) {
		let output = webgpu.surface.get_current_texture().unwrap();
		
		let surface_info = SurfaceInfo {
			width: output.texture.width(),
			height: output.texture.height()
		};

		let tan_half_fov = (self.fov/2.0).min(89.0).tan();
		let cam_fwd = game_world.get_player_forward_vector();
		let cam_plane = cam_fwd.perp() * 0.5;
		let cam_len = cam_plane.length() / tan_half_fov;
		let cam_vec = cam_fwd * cam_len;

		let camera_info = CameraInfo {
			pos: game_world.get_player_position() / game_world.get_grid_size(),
			pos_z: surface_info.height as f32 * 0.5,
			len: cam_len,
			leftmost_ray: cam_vec + cam_plane,
			rightmost_ray: cam_vec - cam_plane
		};

		let tilemap = game_world.get_tilemap();
		let tilemap_data: Vec<_> = tilemap.data.clone().into_iter().map(|ty| match ty {
			TileType::Empty(t1, t2) => glam::ivec2(t1 as i32, t2 as i32),
			TileType::Wall(_) => glam::ivec2(-1, -1)
		}).collect();
		let tilemap_size = glam::uvec2(tilemap.width, tilemap.height);

		webgpu.queue.write_buffer(&self.floorceil_data.surface_info, 0, bytemuck::cast_slice(&[surface_info]));
		webgpu.queue.write_buffer(&self.floorceil_data.camera_info, 0, bytemuck::cast_slice(&[camera_info]));
		webgpu.queue.write_buffer(&self.floorceil_data.tilemap_info, 0, bytemuck::cast_slice(&[tilemap_size]));
		webgpu.queue.write_buffer(&self.floorceil_data.tilemap_info, std::mem::size_of_val(&tilemap_size) as u64, bytemuck::cast_slice(&tilemap_data));

		if let Ok(raycast) = wall_raycast::multiple_raycast(
			&game_world.get_walls(), game_world.get_grid_size(), 
			game_world.get_player_position(), game_world.get_player_forward_vector(), 
			self.fov, self.raycount, 50
		) {
			let raycast_data: Vec<RaycastData> = raycast.into_iter().map(|(d, t, u)| RaycastData {
				distance: d, texid: t, u_offset: u
			}).collect();
			webgpu.queue.write_buffer(&self.wall_data.surface_info_buffer, 0, bytemuck::cast_slice(&[surface_info]));
			webgpu.queue.write_buffer(&self.wall_data.raycast_data_array_buffer, 0, bytemuck::cast_slice(&[raycast_data.len() as u32]));
			webgpu.queue.write_buffer(&self.wall_data.raycast_data_array_buffer, std::mem::size_of::<u32>() as u64, bytemuck::cast_slice(&raycast_data));
		}

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
	

		let mut encoder = webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: Some("firstperson::Renderer::render() floorceil compute pass"),
			..Default::default()
		});

		compute_pass.set_bind_group(0, &self.floorceil_data.bind_groups[0], &[]);
		compute_pass.set_bind_group(1, &self.floorceil_data.bind_groups[1], &[]);
		compute_pass.set_bind_group(2, &self.floorceil_data.bind_groups[2], &[]);

		compute_pass.set_pipeline(&self.floorceil_data.compute_pipelines[0]);
		compute_pass.dispatch_workgroups(output.texture.height(), 1, 1);

		compute_pass.set_pipeline(&self.floorceil_data.compute_pipelines[1]);
		compute_pass.dispatch_workgroups(output.texture.width(), output.texture.height(), 1);

		drop(compute_pass);

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

		render_pass.set_pipeline(&self.floorceil_data.render_pipeline);
		render_pass.set_bind_group(0, &self.floorceil_data.bind_groups[0], &[]);
		render_pass.set_bind_group(1, &self.floorceil_data.bind_groups[1], &[]);
		render_pass.set_bind_group(2, &self.floorceil_data.bind_groups[2], &[]);
		render_pass.draw(0..4, 0..1);

		render_pass.set_pipeline(&self.wall_data.pipeline);
		render_pass.set_bind_group(0, &self.wall_data.bind_groups[0], &[]);
		render_pass.set_bind_group(1, &self.wall_data.bind_groups[1], &[]);
		render_pass.draw(0..4, 0..1);

		drop(render_pass);

		webgpu.queue.submit(Some(encoder.finish()));
		output.present();
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
			fov: PI / 2.0,
			raycount: 300,
			wall_data: wall::Data::new(webgpu), 
			floorceil_data: floorceil::Data::new(webgpu), 
			depth_texture
		}
	}
}

