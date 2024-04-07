use std::collections::{HashMap, HashSet};

use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
	MoveForward, MoveBackward, StrafeLeft, StrafeRight, ToggleMinimap
}

#[derive(Default)]
struct MouseState {
	pub relative_x: f32,
	pub left_pressed: bool,
	pub right_pressed: bool
}

pub struct InputState {
	action_binding: HashMap<Action, HashSet<KeyCode>>,
	key_state: HashMap<KeyCode, u32>,
	mouse_state: MouseState
}

impl InputState {
	const JUST: u32		= 0b01;
	const PRESSED: u32	= 0b10;
}

impl Default for InputState {
	fn default() -> Self {
		let action_binding = HashMap::new();
		let key_state = HashMap::new();
		let mouse_state = MouseState::default();

		let mut input_state = Self {action_binding, key_state, mouse_state};

		input_state.bind_action(Action::MoveForward, KeyCode::KeyW);
		input_state.bind_action(Action::MoveBackward, KeyCode::KeyS);
		input_state.bind_action(Action::StrafeLeft, KeyCode::KeyA);
		input_state.bind_action(Action::StrafeRight, KeyCode::KeyD);
		input_state.bind_action(Action::ToggleMinimap, KeyCode::Tab);

		input_state
	}
}

impl InputState {

	// key binding

	pub fn bind_action(&mut self, action: Action, key: KeyCode) {
		match self.action_binding.get_mut(&action) {
			None => {
				let mut hashset = HashSet::new();
				hashset.insert(key);
				self.action_binding.insert(action, hashset);
			},
			Some(hashset) => { 
				hashset.insert(key);
			}
		}
	}

	// NOTE: works well but add '_' because not used now.
	pub fn _unbind_action(&mut self, action: Action, key: KeyCode) {
		match self.action_binding.get_mut(&action) {
			Some(hashset) => {
				hashset.remove(&key);
			}
			_ => ()
		}
	}

	// NOTE: works well but add '_' because not used now.
	pub fn _clear_action_binding(&mut self, action: Action) {
		self.action_binding.remove(&action);
	}


	// key action press and release

	pub fn set_key_state(&mut self, key: KeyCode, pressed: bool) {
		let pressed_flag = if pressed { Self::PRESSED } else { 0b0 };
		self.key_state.insert(key, Self::JUST | pressed_flag);
	}

	pub fn is_action_pressed(&mut self, action: Action) -> bool {
		match self.action_binding.get(&action) {
			None => false,
			Some(keys) => {
				for key in keys {
					if let Some(state) = self.key_state.get(key) {
						let pressed_flag = state & Self::PRESSED;
						self.key_state.insert(key.clone(), pressed_flag);
						if pressed_flag != 0 {
							return true;
						}
					}
				}
				false
			}
		}
	}

	pub fn is_action_just_pressed(&mut self, action: Action) -> bool {
		match self.action_binding.get(&action) {
			None => false,
			Some(keys) => {
				for key in keys {
					if let Some(state) = self.key_state.get(key) {
						let just_flag = state & Self::JUST;
						let pressed_flag = state & Self::PRESSED;
						self.key_state.insert(key.clone(), pressed_flag);
						if pressed_flag != 0 && just_flag != 0 {
							return true;
						}
					}
				}
				false
			}
		}
	}

	pub fn get_dir_input_vector(&mut self) -> glam::Vec2 {
		let y = if self.is_action_pressed(Action::MoveForward) {1.0} else {0.0} + if self.is_action_pressed(Action::MoveBackward) {-1.0} else {0.0};
		let x = if self.is_action_pressed(Action::StrafeLeft) {-1.0} else {0.0} + if self.is_action_pressed(Action::StrafeRight) {1.0} else {0.0};

		glam::vec2(x, y).try_normalize().unwrap_or_default()
	}


	// mouse

	pub fn add_mouse_x_relative(&mut self, rel: f32) {
		self.mouse_state.relative_x += rel;
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