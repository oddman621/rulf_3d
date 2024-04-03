pub struct ShaderSource;

impl ShaderSource {
	pub const FILLSCREEN: &'static str = include_str!("asset/fillscreen.wgsl");
	pub const FIRSTPERSON_WALL: &'static str = include_str!("asset/firstperson_wall.wgsl");
	pub const FIRSTPERSON_FLOORCEIL: &'static str = include_str!("asset/firstperson_floorceil.wgsl");
	//TODO: Move actor.wgsl str from minimap to ShaderSource
}

pub struct ImageByte;

impl ImageByte {
	pub const ALL_6: &'static [u8] = include_bytes!("asset/all_6.jpg");
}