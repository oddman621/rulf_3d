pub struct ShaderSource;

impl ShaderSource {
	pub const FILLSCREEN: &'static str = include_str!("asset/fillscreen.wgsl");
	pub const FIRSTPERSON_WALL: &'static str = include_str!("asset/firstperson_wall.wgsl");
}

pub struct ImageByte;

impl ImageByte {
	pub const ALL_6: &'static [u8] = include_bytes!("asset/all_6.jpg");
}