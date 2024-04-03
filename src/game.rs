mod collision;
use collision::AABB;


#[derive(Copy, Clone)]
pub enum TileType { Empty(u32, u32), Wall(u32) }

pub struct TileMap {
	pub data: Vec<TileType>,
	pub width: u32,
	pub height: u32,
	pub grid_size: f32
}

impl TileMap {
	pub fn test_tilemap() -> Self {
		const TEST_TILEMAP: [TileType; 64] = [
		TileType::Wall(0), TileType::Wall(1), TileType::Wall(2), TileType::Wall(3), TileType::Wall(3), TileType::Wall(2), TileType::Wall(1), TileType::Wall(0), 
		TileType::Wall(1), TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Wall(1),
		TileType::Wall(2), TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Wall(0), TileType::Wall(1), TileType::Wall(2), TileType::Empty(0, 0),   TileType::Wall(2),
		TileType::Wall(3), TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Wall(3), TileType::Empty(0, 0),   TileType::Wall(3),
		TileType::Wall(3), TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Wall(0), TileType::Wall(1), TileType::Wall(2), TileType::Empty(0, 0),   TileType::Wall(3),
		TileType::Wall(2), TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Wall(2),
		TileType::Wall(1), TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Empty(0, 0),   TileType::Wall(1),
		TileType::Wall(0), TileType::Wall(1), TileType::Wall(2), TileType::Wall(3), TileType::Wall(3), TileType::Wall(2), TileType::Wall(1), TileType::Wall(0) 
		];

		let width = 8;
		let height = 8;

		let data = Vec::<TileType>::from(TEST_TILEMAP);

		TileMap {
			data, width, height,
			grid_size: 100.0
		}
	}
}


impl TileMap {

	fn get_tile(&self, coord: glam::UVec2) -> Option<&TileType> {
		if coord.x >= self.width || coord.y >= self.height {
			return None;
		}
		self.data.get((coord.y * self.width + coord.x) as usize)
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
					TileType::Empty(_, _) => false,
					TileType::Wall(_) => true
				}
		)).collect()
	}
	pub fn circle_collision_check(&self, position: glam::Vec2, radius: f32) -> Option<AABB> {
		for wall_offset in self.get_near_walls_coord_from(position).into_iter().map(|f| f.as_vec2() * self.grid_size) {
			let aabb = AABB::from_rect(wall_offset, self.grid_size, self.grid_size);
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
	//doors: BtreeMap<[u32;2], Door>
	//statics: BtreeMap<[u32;2], Static>
	//enemies: BtreeMap<[f32;2], Enemy>
}

impl GameWorld {
	pub fn get_tilemap(&self) ->&TileMap {
		&self.tilemap
	}

	pub fn test_gameworld() -> Self {
		GameWorld {
			tilemap: TileMap::test_tilemap(),
			player: Object { angle: 0.0, position: glam::vec2(200.0, 200.0), radius: 25.0 },
		}
	}
	pub fn get_walls(&self) -> std::collections::HashMap<glam::UVec2, u32> {
		self.tilemap.data.iter().enumerate().filter_map(|(i, ty)| match ty {
			TileType::Empty(_, _) => None,
			TileType::Wall(id) => Some((
				glam::uvec2(i as u32 % self.tilemap.width, i as u32 / self.tilemap.width), 
				id.clone()
			))
		}).collect()
	}
	pub fn get_grid_size(&self) -> f32 {
		self.tilemap.grid_size
	}
	pub fn actors_position_angle_flatten(&self) -> Vec<[f32; 3]> {
		Vec::<[f32; 3]>::from([[self.player.position.x, self.player.position.y, self.player.angle]]) //TODO: actors_position, actors_angle 모두 현재는 player만 담긴 Vector를 반환함. 추후 player 포함 enemies 목록까지 반환하도록 구현.
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
		glam::Vec2::from_angle(self.player.angle)
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
	let walls = gameworld.get_walls();
	assert!(walls.get(&glam::uvec2(0, 0)).is_some());
	assert!(walls.get(&glam::uvec2(1, 1)).is_none());
	assert!(walls.get(&glam::uvec2(7, 7)).is_some());
}