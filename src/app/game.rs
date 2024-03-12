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
	pub data: Vec<TileType>,
	pub width: u32,
	pub height: u32,
	pub grid_size: glam::Vec2
}

impl TileMap {
	pub fn get_tile(&self, x: u32, y: u32) -> Option<&TileType> {
		if x >= self.width || y >= self.width {
			return None
		}

		let idx = y * self.width + x;

		if idx + 1 >= self.width * self.height {
			return None
		}

		Some(&self.data[idx as usize])
	}
}

pub struct GameWorld {
	tilemap: TileMap,
	player: Player,
	//sky: Sky,
	//ground_color: glam::Vec3,
	//show_minimap: bool,
	//paused: bool,
	//doors: BtreeMap<[f32;2], Door>
	//statics: BtreeMap<[f32;2], Static>
	//enemies: BtreeMap<[f32;2], Enemy>
	//gui: GUI
}

//wall_renderer: wall offsets(GameScene), grid size(GameScene)
//actor_renderer: actor pos(GameScene), actor angle(GameScene)
impl GameWorld {
	pub fn walls_offset(&self) ->  std::collections::HashSet<glam::UVec2> {
		self.tilemap.data.iter().enumerate().filter_map(|(idx, ty)| match ty {
			TileType::Empty => None,
			TileType::Wall(_) => Some(glam::uvec2(idx as u32 % self.tilemap.width, idx as u32 / self.tilemap.width))
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
		self.player.position = pos;
	}
	pub fn translate_player(&mut self, wishvec: glam::Vec2) {
		self.player.position += wishvec;
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
	angle: f32
}

pub fn create_test_gameworld() -> GameWorld {
	GameWorld {
		tilemap: create_test_tilemap(),
		player: Player { angle: 0.0, position: glam::vec2(200.0, 200.0) },
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

	TileMap {
		data: Vec::<TileType>::from(TEST_TILEMAP),
		width: 8,
		height: 8,
		grid_size: glam::vec2(100.0, 100.0)
	}
}