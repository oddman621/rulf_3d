mod input;

mod game;

mod renderer;

use crate::framework::{FrameworkLoop, InputEvent};

pub struct GameApp {
	scene: game::GameWorld,
	input: input::UserInput,
	minimap_renderer: renderer::MiniMapRenderer
}

impl InputEvent for GameApp {
	fn keyboard_input(&mut self, keycode: winit::keyboard::KeyCode, state: winit::event::ElementState) {
		self.input.keyboard_input(keycode, state);
	}
	fn mouse_move_input(&mut self, position: glam::Vec2, relative: glam::Vec2) {
		self.input.mouse_move_input(position, relative);
	}
	fn mouse_button_input(&mut self, button: winit::event::MouseButton, state: winit::event::ElementState) {
		self.input.mouse_button_input(button, state);
	}
}

impl FrameworkLoop for GameApp {
	fn init(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
		GameApp { 
			scene: game::create_test_gameworld(), 
			input: input::create_test_user_input(), 
			minimap_renderer: renderer::MiniMapRenderer::new(device, queue, surface_format) 
		}
	}
	fn process(&mut self, _delta: f64) {
		let _dir_input_vec = self.input.get_dir_input_vector();
		let _mouse_rel_x = self.input.get_mouse_relative_x();
	}
	fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface) {
		// <2D MiniMap Rendering>
		//common: view projection(self)
		//wall_renderer: wall offsets(GameScene), grid size(GameScene)
		//actor_renderer: actor pos(GameScene), actor angle(GameScene)

		let cam_pos = glam::Mat4::from_translation(glam::vec3(200.0, 200.0, 0.0));
		let view = cam_pos.inverse();
		let proj = glam::Mat4::orthographic_lh(-400.0, 400.0, -300.0, 300.0, -0.001, 1.0001);
		let viewproj = proj * view; //Must Change
		
		// for wall rendering
		let wall_offsets: Vec<glam::UVec2> = self.scene.walls_offset().into_iter().collect();
		let gridsize = self.scene.tile_grid_size();

		// for actors rendering
		let actors_pos = self.scene.actors_position();
		let actors_angle = self.scene.actors_angle();
		let actor_color = glam::vec4(0.3, 0.2, 0.1, 1.0);

		self.minimap_renderer.draw(device, queue, surface, &wgpu::Color{r:0.1, g:0.2, b:0.3, a:1.0}, &viewproj, wall_offsets.as_slice(), &gridsize, actors_pos.as_slice(), actors_angle.as_slice(), &actor_color);
	}
}



