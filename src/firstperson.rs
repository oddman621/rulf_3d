use std::f32::consts::PI;
use crate::{
	game::{GameWorld, TileType}, 
	webgpu::{WebGPU, WebGPUDevice, WebGPUSurface},
	asset::AssetServer
};

mod wall;
mod floorceil;

const CAMERA_NEAR: f32 = 0.0;
const CAMERA_FAR: f32 = 100.0;

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
	depth: f32,
	texid: u32,
	u_offset: f32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ScanlineData {
	depth: f32,
	floor: glam::Vec2,
	floor_step: glam::Vec2
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct FloorCeilCameraInfo {
	pos: glam::Vec2,
	pos_z: f32,
	len: f32,
	leftmost_ray: glam::Vec2,
	rightmost_ray: glam::Vec2,
	near: f32,
	far: f32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct WallCameraInfo {
	tiledpos: glam::Vec2,
	dirvec: glam::Vec2,
	plane: glam::Vec2,
	near: f32,
	far: f32
}

pub struct Renderer {
	pub fov: f32,
	floorceil_data: floorceil::Data,
	wall_data: wall::Data,
	depth_texture: wgpu::Texture
}

impl Renderer {
	pub fn render(&mut self, webgpu: &WebGPU, game_world: &GameWorld, clear_color: &wgpu::Color) {
		let output = webgpu.get_surface().get_current_texture().unwrap();
		
		let surface_info = SurfaceInfo {
			width: output.texture.width(),
			height: output.texture.height()
		};

		let tan_half_fov = (self.fov/2.0).min(89.0f32.to_radians()).tan();
		let cam_dir = game_world.get_player_forward_vector();
		let cam_plane = cam_dir.perp() * 0.5;
		let cam_len = cam_plane.length() * 2.0 / tan_half_fov; // BUG: Coincidence Problem. Fixed with magic number 2.0, still not solved completely.
		let cam_vec = cam_dir * cam_len;
		let cam_pos = game_world.get_player_position() / game_world.get_grid_size();

		let floorceil_camera_info = FloorCeilCameraInfo {
			pos: cam_pos,
			pos_z: surface_info.height as f32 * 0.5,
			len: cam_len,
			leftmost_ray: cam_vec + cam_plane,
			rightmost_ray: cam_vec - cam_plane,
			near: CAMERA_NEAR,
			far: CAMERA_FAR
		};

		let wall_camera_info = WallCameraInfo {
			tiledpos: cam_pos,
			dirvec: cam_dir * cam_plane.length() / tan_half_fov,
			plane: cam_plane,
			near: CAMERA_NEAR,
			far: CAMERA_FAR
		};

		let tilemap = game_world.get_tilemap();
		let tilemap_empty_data: Vec<_> = tilemap.data.clone().into_iter().map(|ty| match ty {
			TileType::Empty(t1, t2) => glam::ivec2(t1 as i32, t2 as i32),
			TileType::Wall(_) => glam::ivec2(-1, -1)
		}).collect();
		let tilemap_wall_data: Vec<_> = tilemap.data.clone().into_iter().map(|ty| match ty {
			TileType::Empty(_, _) => -1,
			TileType::Wall(id) => id as i32
		}).collect();
		let tilemap_size = glam::uvec2(tilemap.width, tilemap.height);

		let (device, queue) = webgpu.get_device();

		queue.write_buffer(&self.floorceil_data.surface_info, 0, bytemuck::cast_slice(&[surface_info]));
		queue.write_buffer(&self.floorceil_data.camera_info, 0, bytemuck::cast_slice(&[floorceil_camera_info]));
		queue.write_buffer(&self.floorceil_data.tilemap_info, 0, bytemuck::cast_slice(&[tilemap_size]));
		queue.write_buffer(&self.floorceil_data.tilemap_info, std::mem::size_of_val(&tilemap_size) as u64, bytemuck::cast_slice(&tilemap_empty_data));

		queue.write_buffer(&self.wall_data.surface_info_buffer, 0, bytemuck::cast_slice(&[surface_info]));
		queue.write_buffer(&self.wall_data.camera_info, 0, bytemuck::cast_slice(&[wall_camera_info]));
		queue.write_buffer(&self.wall_data.tilemap_data, 0, bytemuck::cast_slice(&[tilemap_size]));
		queue.write_buffer(&self.wall_data.tilemap_data, std::mem::size_of_val(&tilemap_size) as u64, bytemuck::cast_slice(&tilemap_wall_data));
		queue.write_buffer(&self.wall_data.raycast_data_array_buffer, 0, bytemuck::cast_slice(&[surface_info.width]));

		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		if self.depth_texture.size() != size {
			self.depth_texture.destroy();
			self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
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
	
		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
			label: Some("firstperson::Renderer::render() wall/floorceil compute pass"),
			..Default::default()
		});

		compute_pass.set_bind_group(0, &self.wall_data.compute_bind_group, &[]);
		compute_pass.set_pipeline(&self.wall_data.compute_pipeline);
		compute_pass.dispatch_workgroups(surface_info.width, 1, 1);

		compute_pass.set_bind_group(0, &self.floorceil_data.bind_groups[0], &[]);
		compute_pass.set_bind_group(1, &self.floorceil_data.bind_groups[1], &[]);
		compute_pass.set_bind_group(2, &self.floorceil_data.bind_groups[2], &[]);

		compute_pass.set_pipeline(&self.floorceil_data.compute_pipelines[0]);
		compute_pass.dispatch_workgroups(output.texture.height(), 1, 1);

		// NOTE: No use this compute pass because gpu usage is too high.
		// The compute code will be done in fragment code.
		// compute_pass.set_pipeline(&self.floorceil_data.compute_pipelines[1]);
		// compute_pass.dispatch_workgroups(output.texture.width(), output.texture.height(), 1);

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

		render_pass.set_pipeline(&self.wall_data.render_pipeline);
		render_pass.set_bind_group(0, &self.wall_data.render_bind_groups[0], &[]);
		render_pass.set_bind_group(1, &self.wall_data.render_bind_groups[1], &[]);
		render_pass.draw(0..4, 0..1);

		drop(render_pass);

		queue.submit(Some(encoder.finish()));
		output.present();
	}
	pub fn new(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
		let (device, _) = webgpu.get_device();
		let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
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
			// BUG: Gap Problem. There's a gap between floorceils and walls. Both leftside and rightside has gaps but the rightside seems bigger.
			// Fixing by magic number. Why does fov value influence floorceil's height?
			fov: PI / 2.3,
			wall_data: wall::Data::new(webgpu, asset_server), 
			floorceil_data: floorceil::Data::new(webgpu, asset_server), 
			depth_texture
		}
	}
}

