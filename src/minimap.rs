use crate:: {
	webgpu::{WebGPU, WebGPUDevice, WebGPUSurface},
	game::GameWorld,
	asset::AssetServer
};

mod wall;
mod actor;

pub struct Renderer {
	wall_render: wall::WallRender,
	actor_render: actor::ActorRender,
}

impl Renderer {
	pub fn new(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
		Self { 
			wall_render: wall::WallRender::new(webgpu, asset_server), 
			actor_render: actor::ActorRender::new(webgpu, asset_server),
		}
	}
}

impl Renderer {
	pub fn render(&mut self, webgpu: &WebGPU, game_world: &GameWorld, clear_color: &wgpu::Color) {
		// Convert game data to renderer specific
		let cam_pos = glam::Mat4::from_translation(game_world.get_player_position().extend(0.0));
		let cam_rot = glam::Mat4::IDENTITY;//glam::Mat4::from_rotation_z(-std::f32::consts::FRAC_PI_2 + self.scene.get_player_angle());
		let view = cam_rot.inverse() * cam_pos.inverse();
		let proj = glam::Mat4::orthographic_lh(-400.0, 400.0, -300.0, 300.0, -0.001, 1.0001);
		let viewproj = proj * view;
		
		// for wall rendering
		let walls: Vec<u32> = game_world.get_walls().into_iter().flat_map(|(uvec, id)| 
			{[uvec.x, uvec.y, id]}).collect();
		let gridsize = game_world.get_grid_size();

		// for actors rendering
		let actor_size = 50.0f32;
		let actors_pos_ang = game_world.actors_position_angle_flatten();
		let actor_color = glam::vec4(0.3, 0.2, 0.1, 1.0);

		let (device, queue) = webgpu.get_device();
		queue.write_buffer(&self.wall_render.instb, 0, bytemuck::cast_slice(walls.as_slice()));
		queue.write_buffer(&self.wall_render.viewproj_ub, 0, bytemuck::cast_slice(&[viewproj]));
		queue.write_buffer(&self.wall_render.gridsize_ub, 0, bytemuck::cast_slice(&[gridsize]));
		self.wall_render.instb_len = walls.len() as u32 / 3;

		queue.write_buffer(&self.actor_render.viewproj_ub, 0, bytemuck::cast_slice(&[viewproj]));
		queue.write_buffer(&self.actor_render.actorsize_ub, 0, bytemuck::cast_slice(&[actor_size]));
		queue.write_buffer(&self.actor_render.color_ub, 0, bytemuck::cast_slice(&[actor_color]));
		queue.write_buffer(&self.actor_render.instb, 0, bytemuck::cast_slice(actors_pos_ang.as_slice()));
		self.actor_render.instb_len = actors_pos_ang.len() as u32;


		let output = webgpu.get_surface().get_current_texture().unwrap();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Renderer::draw() clear color"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(*clear_color),
					store: wgpu::StoreOp::Store
				}
			})],
			..Default::default()
		});

		render_pass.set_pipeline(&self.wall_render.pipeline);
		render_pass.set_bind_group(0, &self.wall_render.bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.wall_render.vb.slice(..));
		render_pass.set_vertex_buffer(1, self.wall_render.instb.slice(..));
		render_pass.draw(0..4, 0..self.wall_render.instb_len);

		render_pass.set_pipeline(&self.actor_render.pipeline);
		render_pass.set_bind_group(0, &self.actor_render.bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.actor_render.vb.slice(..));
		render_pass.set_vertex_buffer(1, self.actor_render.instb.slice(..));
		render_pass.draw(0..3, 0..self.actor_render.instb_len as u32);
		
		drop(render_pass);
		queue.submit(Some(encoder.finish()));
		output.present();
	}
}


