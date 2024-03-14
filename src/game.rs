//
// Collision
//

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

//
// Collision End
//


#[derive(Copy, Clone)]
enum TileType { Empty, Wall(u32) }

enum Sky {
	/*Texture(u32), hide(comment) because of unused, for now*/ SolidColor(glam::Vec3)
}

impl Default for Sky {
	fn default() -> Self {
		Sky::SolidColor(glam::vec3(0.1, 0.3, 0.2))
	}
}

struct TileMap {
	pub data: std::collections::BTreeMap<[u32; 2], TileType>,
	pub width: u32,
	pub height: u32,
	pub grid_size: glam::Vec2
}

impl TileMap {
	pub fn test_tilemap() -> Self {
		const TEST_TILEMAP: [TileType; 64] = [
		TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), 
		TileType::Wall(0), TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Wall(0),
		TileType::Wall(0), TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Wall(0),
		TileType::Wall(0), TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Wall(0),
		TileType::Wall(0), TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Wall(0),
		TileType::Wall(0), TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Wall(0),
		TileType::Wall(0), TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Empty,   TileType::Wall(0),
		TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0), TileType::Wall(0) 
		];

		let width = 8;
		let height = 8;
		let test_tilemap = TEST_TILEMAP.iter().enumerate().filter_map(|(idx, ty)|
			Some(([idx as u32 % width, idx as u32 / width], *ty))
		);

		let data = std::collections::BTreeMap::<[u32; 2], TileType>::from_iter(test_tilemap);

		TileMap {
			data, width, height,
			grid_size: glam::vec2(100.0, 100.0)
		}
	}
}


impl TileMap {
	fn get_tile(&self, coord: glam::UVec2) -> Option<&TileType> {
		if coord.x >= self.width || coord.y >= self.height {
			return None;
		}
		self.data.get(&[coord.x, coord.y])
	}

	fn point_to_tile_coord(&self, point: glam::Vec2) -> glam::UVec2 {
		(point / self.grid_size).round().as_uvec2()
	}
	
	fn get_near_walls_coord_from(&self, point: glam::Vec2) -> Vec<glam::UVec2> {
		let mut retval = Vec::<glam::UVec2>::new();
		if point.x < 0.0 || point.y < 0.0 {
			return retval;
		}
		let glam::UVec2 {x, y} = self.point_to_tile_coord(point);
		
		//(x,y) (x-1,y) (x,y-1) (x-1,y-1)
		retval.push(glam::uvec2(x, y));
		//NOTE: if문 생략해도 될까?
		if x > 0 {
			retval.push(glam::uvec2(x-1, y));
		}
		if y > 0 {
			retval.push(glam::uvec2(x, y-1));
		}
		if x > 0 && y > 0 {
			retval.push(glam::uvec2(x-1, y-1))
		}

		retval.into_iter().filter(|p| 
			self.get_tile(*p).is_some_and(|f| 
				match f {
					TileType::Empty => false,
					TileType::Wall(_) => true
				}
		)).collect()
	}
	pub fn circle_collision_check(&self, position: glam::Vec2, radius: f32) -> Option<AABB> {
		for wall_offset in self.get_near_walls_coord_from(position).into_iter().map(|f| f.as_vec2() * self.grid_size) {
			let aabb = AABB::from_rect(wall_offset, self.grid_size.x, self.grid_size.y);
			if aabb.circle_collision_check(position, radius) {
				return Some(aabb);
			}
		}
		None
	}
}

pub struct GameWorld {
	tilemap: TileMap,
	player: Object,
	//sky: Sky,
	//ground_color: glam::Vec3,
	//show_minimap: bool,
	//paused: bool,
	//doors: BtreeMap<[f32;2], Door>
	//statics: BtreeMap<[f32;2], Static>
	//enemies: BtreeMap<[f32;2], Enemy>
	//gui: GUI
}

impl GameWorld {
	pub fn test_gameworld() -> Self {
		GameWorld {
			tilemap: TileMap::test_tilemap(),
			player: Object { angle: 0.0, position: glam::vec2(200.0, 200.0), radius: 25.0 },
		}
	}
	pub fn walls_offset(&self) ->  std::collections::HashSet<glam::UVec2> {
		self.tilemap.data.iter().filter_map(|(coord, ty)| match ty {
			TileType::Empty => None,
			TileType::Wall(_) => Some(glam::uvec2(coord[0], coord[1]))
		}).collect()
	}
	pub fn tile_grid_size(&self) -> glam::Vec2 {
		self.tilemap.grid_size
	}
	pub fn actors_position(&self) -> Vec<glam::Vec2> {
		Vec::<glam::Vec2>::from([self.player.position]) //TODO: actors_position, actors_angle 모두 현재는 player만 담긴 Vector를 반환함. 추후 player 포함 enemies 목록까지 반환하도록 구현.
	}
	pub fn actors_angle(&self) -> Vec<f32> {
		Vec::<f32>::from([self.player.angle])
	}
	pub fn get_player_position(&self) -> glam::Vec2 {
		self.player.position
	}
	pub fn set_player_position(&mut self, pos: glam::Vec2) {
		match self.tilemap.circle_collision_check(pos, self.player.radius) {
			None => self.player.position = pos,
			Some(_) => { // Try move along axis
				let wishvec = pos - self.player.position;
				let proj_x = wishvec.project_onto(glam::Vec2::X);
				let proj_y = wishvec.project_onto(glam::Vec2::Y);

				if self.tilemap.circle_collision_check(self.player.position + proj_x, self.player.radius).is_none() {
					self.player.position += proj_x;
				}
				else if self.tilemap.circle_collision_check(self.player.position + proj_y, self.player.radius).is_none() {
					self.player.position += proj_y;
				}
			}
		}
	}
	pub fn translate_player(&mut self, wishvec: glam::Vec2) {
		self.set_player_position(self.player.position + wishvec);
	}
	pub fn rotate_player(&mut self, wishang: f32) {
		self.player.angle += wishang;
	}
	pub fn get_player_forward_vector(&self) -> glam::Vec2 {
		glam::Vec2::from_angle(self.player.angle).rotate(glam::Vec2::X)
	}
}


#[derive(Default)]
struct Object {
	position: glam::Vec2,
	angle: f32,
	radius: f32,
}




#[test]
fn test_get_near_walls() {
	let tilemap = TileMap::test_tilemap();

	assert!(tilemap.circle_collision_check(glam::vec2(60.0, 60.0), 50.0).is_some());
}

#[test]
fn gameworld_walls_offset_test() {
	let gameworld = GameWorld::test_gameworld();
	let walls_offset = gameworld.walls_offset();
	assert!(walls_offset.get(&glam::uvec2(0, 0)).is_some());
	assert!(walls_offset.get(&glam::uvec2(1, 1)).is_none());
	assert!(walls_offset.get(&glam::uvec2(7, 7)).is_some());
}