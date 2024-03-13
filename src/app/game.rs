mod collision;

#[derive(Copy, Clone)]
enum TileType { Empty, Wall(u32) }

enum Sky {
	Texture(u32), SolidColor(glam::Vec3)
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

#[test]
fn test_get_near_walls() {
	let tilemap = create_test_tilemap();

	assert!(tilemap.circle_collision_test(glam::vec2(60.0, 60.0), 50.0) == true);
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
	
	/*
	아이디어: 위치값 (f32, f32)를 반올림한 값은
	해당 위치타일의 꼭짓점 4개중 1개에 붙게됨. 
	그 꼭지점에 접한 4개의 타일에 대해 wall 체크 및 콜리전 검사 수행.
	*/
	fn get_near_walls_coord_from(&self, point: glam::Vec2) -> Vec<glam::UVec2> {
		let mut retval = Vec::<glam::UVec2>::new();
		if point.x < 0.0 || point.y < 0.0 {
			return retval;
		}
		let glam::UVec2 {x, y} = self.point_to_tile_coord(point);
		
		//(x,y) (x-1,y) (x,y-1) (x-1,y-1)
		retval.push(glam::uvec2(x, y)); // NOTE: 포함시킬 필요가 있나?
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
	pub fn circle_collision_test(&self, position: glam::Vec2, radius: f32) -> bool {
		for wall_offset in self.get_near_walls_coord_from(position).into_iter().map(|f| f.as_vec2() * self.grid_size) {
			let aabb = collision::AABB::from_rect(wall_offset, self.grid_size.x, self.grid_size.y);
			if aabb.circle_collision(position, radius) {
				return true;
			}
		}
		false
	}
}

pub struct GameWorld {
	tilemap: TileMap,
	player: Player, //TODO: Traitize Player to Actor, use it both player and enemies
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
		if !self.tilemap.circle_collision_test(pos, self.player.size_radius) {
			self.player.position = pos;
		}
	}
	pub fn translate_player(&mut self, wishvec: glam::Vec2) {
		self.set_player_position(self.player.position + wishvec);
	}
	pub fn get_player_angle(&self) -> f32 {
		self.player.angle
	}
	pub fn set_player_angle(&mut self, ang: f32) {
		self.player.angle = ang;
	}
	pub fn rotate_player(&mut self, wishang: f32) {
		self.player.angle += wishang;
	}
	pub fn get_player_forward_vector(&self) -> glam::Vec2 {
		glam::Vec2::from_angle(self.player.angle).rotate(glam::Vec2::X)
	}
}


#[test]
fn gameworld_walls_offset_test() {
	let gameworld = create_test_gameworld();
	let walls_offset = gameworld.walls_offset();
	assert!(walls_offset.get(&glam::uvec2(0, 0)).is_some());
	assert!(walls_offset.get(&glam::uvec2(1, 1)).is_none());
	assert!(walls_offset.get(&glam::uvec2(7, 7)).is_some());
}

#[derive(Default)]
struct Player {
	position: glam::Vec2,
	angle: f32,
	size_radius: f32,
}

pub fn create_test_gameworld() -> GameWorld {
	GameWorld {
		tilemap: create_test_tilemap(),
		player: Player { angle: 0.0, position: glam::vec2(200.0, 200.0), size_radius: 25.0 },
	}
}

fn create_test_tilemap() -> TileMap {
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