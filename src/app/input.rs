use winit::keyboard::KeyCode;
use crate::framework::InputEvent;
pub struct UserInput { // do some with Scene
	move_forward: KeyCode,
	move_backward: KeyCode,
	strafe_left: KeyCode,
	strafe_right: KeyCode,
	key_pressed_states: std::collections::HashMap<KeyCode, bool>,
	mouse_position: glam::Vec2,
	mouse_relative: glam::Vec2,
	mouse_left_pressed: bool,
	mouse_right_pressed: bool
}

pub fn create_test_user_input() -> UserInput {
	let move_forward = KeyCode::KeyW;
	let move_backward = KeyCode::KeyS;
	let strafe_left = KeyCode::KeyA;
	let strafe_right = KeyCode::KeyD;
	let mut key_pressed_states = std::collections::HashMap::new();
	key_pressed_states.insert(move_forward, false);
	key_pressed_states.insert(move_backward, false);
	key_pressed_states.insert(strafe_left, false);
	key_pressed_states.insert(strafe_right, false);

	UserInput {
		move_forward, move_backward, strafe_left, strafe_right, key_pressed_states,
		mouse_position: glam::Vec2::default(),
		mouse_relative: glam::Vec2::ZERO,
		mouse_left_pressed: false,
		mouse_right_pressed: false
	}
}

impl InputEvent for UserInput {
	fn keyboard_input(&mut self, keycode: winit::keyboard::KeyCode, state: winit::event::ElementState) {
		let _ = self.key_pressed_states.insert(keycode, if state.is_pressed() { true } else { false });
	}
	fn mouse_button_input(&mut self, button: winit::event::MouseButton, state: winit::event::ElementState) {
		match button {
			winit::event::MouseButton::Left => self.mouse_left_pressed = state.is_pressed(),
			winit::event::MouseButton::Right => self.mouse_right_pressed = state.is_pressed(),
			_ => ()
		};
	}
	fn mouse_move_input(&mut self, position: glam::Vec2, relative: glam::Vec2) {
		self.mouse_position = position; self.mouse_relative = relative;
	}
}

impl UserInput {
	fn is_key_pressed(&self, keycode: &KeyCode) -> bool {
		self.key_pressed_states.get(keycode).cloned().unwrap_or(false)
	}
	pub fn take_mouse_relative_x(&mut self) -> f32 {
		let retval = self.mouse_relative.x;
		self.mouse_relative = glam::Vec2::ZERO;
		retval
	}
	pub fn get_dir_input_vector(&self) -> glam::Vec2 {
		let y = if self.is_key_pressed(&self.move_forward) {1.0} else {0.0} + if self.is_key_pressed(&self.move_backward) {-1.0} else {0.0};
		let x = if self.is_key_pressed(&self.strafe_left) {-1.0} else {0.0} + if self.is_key_pressed(&self.strafe_right) {1.0} else {0.0};

		glam::vec2(x, y).try_normalize().unwrap_or_default()
	}
}
