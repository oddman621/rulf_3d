





pub struct AABB {
	pub left: f32, pub right: f32, pub top: f32, pub bottom: f32
}

impl AABB {
	pub fn from_rect(position: glam::Vec2, width: f32, height: f32) -> Self {
		Self {
			left: position.x,
			right: position.x + width,
			top: position.y + height,
			bottom: position.y
		}
	}

	pub fn circle_collision_check(&self, position: glam::Vec2, radius: f32) -> bool {
		let test_x;
		let test_y;
		
		if position.x < self.left {
			 test_x = self.left;
		} else if position.x > self.right {
			test_x = self.right;
		} else {
			test_x = position.x;
		}
	
		if position.y > self.top {
			test_y = self.top;
		} else if position.y < self.bottom {
			test_y = self.bottom;
		} else {
			test_y = position.y;
		}

		let closest_dist_squared = glam::vec2(position.x-test_x, position.y-test_y).length_squared();
		let radius_squared = radius.powi(2);
		closest_dist_squared <= radius_squared // int squared comparison instead of sqrt for performance
	}
}