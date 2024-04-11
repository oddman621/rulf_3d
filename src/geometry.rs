#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: glam::Vec3,
	pub color: glam::Vec3,
	pub uv: glam::Vec2
}

pub const QUAD: [Vertex; 4] = [
	Vertex { position: glam::vec3(1.0, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 1.0) },
	Vertex { position: glam::vec3(1.0, 1.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.0) },
	Vertex { position: glam::vec3(0.0, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) },
	Vertex { position: glam::vec3(0.0, 1.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) }
];

pub const ACTOR_TRIANGLE: [Vertex; 3] = [
	Vertex { position: glam::vec3(0.5, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.5) },
	Vertex { position: glam::vec3(-0.5, 0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) },
	Vertex { position: glam::vec3(-0.5, -0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) }
];