pub struct ShaderSource;

impl ShaderSource {
	pub const FILLSCREEN: &'static str = include_str!("asset/fillscreen.wgsl");
	pub const FIRSTPERSON_WALL_COMPUTE: &'static str = include_str!("asset/firstperson_wall_compute.wgsl");
	pub const FIRSTPERSON_WALL_FRAG: &'static str = include_str!("asset/firstperson_wall_frag.wgsl");
	pub const FIRSTPERSON_FLOORCEIL: &'static str = include_str!("asset/firstperson_floorceil.wgsl");
	pub const MINIMAP_ACTOR: &'static str = include_str!("asset/minimap_actor.wgsl");
	pub const MINIMAP_WALL: &'static str = include_str!("asset/minimap_wall.wgsl");
}

pub struct ImageByte;

impl ImageByte {
	pub const ALL_6: &'static [u8] = include_bytes!("asset/all_6.jpg");
}