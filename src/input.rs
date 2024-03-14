use std::collections::HashMap;

use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	MoveForward, MoveBackward, StrafeLeft, StrafeRight//, TurnLeft, TurnRight
}

#[derive(Default)]
struct MouseState {
	pub relative_x: f32,
	pub left_pressed: bool,
	pub right_pressed: bool
}

pub struct InputState {
	key_binding: HashMap<KeyCode, Action>,
	action_state: HashMap<Action, bool>,
	mouse_state: MouseState
}

impl Default for InputState {
	fn default() -> Self {
		let key_binding = HashMap::new();
		let action_state = HashMap::new();
		let mouse_state = MouseState::default();

		let mut input_state = Self {key_binding, action_state, mouse_state};

		input_state.bind_key(KeyCode::KeyW, Action::MoveForward);
		input_state.bind_key(KeyCode::KeyS, Action::MoveBackward);
		input_state.bind_key(KeyCode::KeyA, Action::StrafeLeft);
		input_state.bind_key(KeyCode::KeyD, Action::StrafeRight);

		input_state
	}
}

impl InputState {
	pub fn bind_key(&mut self, key: KeyCode, action: Action) {
		self.key_binding.insert(key, action);
	}

	pub fn set_key_state(&mut self, key: KeyCode, pressed: bool) {
		if let Some(action) = self.key_binding.get(&key) {
			self.set_action_state(*action, pressed);
		}
	}

	pub fn set_action_state(&mut self, action: Action, pressed: bool) {
		self.action_state.insert(action, pressed);
	}

	pub fn is_action_pressed(&self, action: Action) -> bool {
		self.action_state.get(&action).unwrap_or(&false).clone()
	}

	pub fn get_dir_input_vector(&self) -> glam::Vec2 {
		let y = if self.is_action_pressed(Action::MoveForward) {1.0} else {0.0} + if self.is_action_pressed(Action::MoveBackward) {-1.0} else {0.0};
		let x = if self.is_action_pressed(Action::StrafeLeft) {-1.0} else {0.0} + if self.is_action_pressed(Action::StrafeRight) {1.0} else {0.0};

		glam::vec2(x, y).try_normalize().unwrap_or_default()
	}

	pub fn set_mouse_x_relative(&mut self, rel: f32) {
		self.mouse_state.relative_x = rel;
	}
	pub fn take_mouse_x_relative(&mut self) -> f32 {
		let rel = self.mouse_state.relative_x;
		self.mouse_state.relative_x = 0.0;
		rel
	}

	pub fn set_mouse_left_pressed(&mut self, pressed: bool) {
		self.mouse_state.left_pressed = pressed;
	}
	pub fn set_mouse_right_pressed(&mut self, pressed: bool) {
		self.mouse_state.right_pressed = pressed;
	}
}