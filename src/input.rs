use std::collections::HashMap;

use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	MoveForward, MoveBackward, StrafeLeft, StrafeRight, TurnLeft, TurnRight
}

pub struct InputState {
	key_binding: HashMap<KeyCode, Action>,
	action_state: HashMap<Action, bool>,
	mouse_x_relative: f32
}

impl Default for InputState {
	fn default() -> Self {
		let mut key_binding = HashMap::new();
		let mut action_state = HashMap::new();
		let mouse_x_relative = 0.0;

		key_binding.insert(KeyCode::KeyW, Action::MoveForward);
		key_binding.insert(KeyCode::KeyS, Action::MoveBackward);
		key_binding.insert(KeyCode::KeyA, Action::StrafeLeft);
		key_binding.insert(KeyCode::KeyD, Action::StrafeRight);

		Self {key_binding, action_state, mouse_x_relative}
	}
}

impl InputState {
	pub fn bind_key(&mut self, key: KeyCode, action: Action) {
		self.key_binding.insert(key, action);
	}

	pub fn set_key_state(&mut self, key: KeyCode, pressed: bool) {
		if let Some(action) = self.key_binding.get(&key) {
			self.action_state.insert(*action, pressed);
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
		self.mouse_x_relative = rel;
	}
	pub fn take_mouse_x_relative(&mut self) -> f32 {
		let rel = self.mouse_x_relative;
		self.mouse_x_relative = 0.0;
		rel
	}
}