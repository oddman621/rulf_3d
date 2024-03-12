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

	let width = 8;
	let height = 8;
	let test_tilemap = TEST_TILEMAP.iter().enumerate().filter_map(|(idx, ty)|
		Some(([idx as u32 % width, idx as u32 / width], *ty))
	);

	let _debug: Vec<([u32; 2], TileType)> = test_tilemap.clone().collect();

	let data = std::collections::BTreeMap::<[u32; 2], TileType>::from_iter(test_tilemap);

	TileMap {
		data, width, height,
		grid_size: glam::vec2(100.0, 100.0)
	}
}