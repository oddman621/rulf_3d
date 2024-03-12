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
		let wall_offsets: Vec<glam::UVec2> = self.scene.walls_offset().into_iter().collect();

		let viewproj = glam::Mat4::orthographic_lh(-400.0, 400.0, -300.0, 300.0, -1.0, 100.0); //Must Change
		let gridsize = self.scene.tile_grid_size();
		self.minimap_renderer.draw_walls(device, queue, surface, wall_offsets.as_slice(), &viewproj, &gridsize);
	}
}



