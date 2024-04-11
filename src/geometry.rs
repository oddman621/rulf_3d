#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: glam::Vec3,
	pub color: glam::Vec3,
	pub uv: glam::Vec2
}

impl Vertex {
	pub const VERT_ATTR: [wgpu::VertexAttribute;3] = [
		wgpu::VertexAttribute {
			format: wgpu::VertexFormat::Float32x3,
			offset: 0,
			shader_location: 0
		},
		wgpu::VertexAttribute {
			format: wgpu::VertexFormat::Float32x3,
			offset: std::mem::size_of::<glam::Vec3>() as u64,
			shader_location: 1
		},
		wgpu::VertexAttribute {
			format: wgpu::VertexFormat::Float32x2,
			offset: std::mem::size_of::<glam::Vec3>() as u64 * 2,
			shader_location: 2
		}
	];
}

pub const QUAD_VERT: [Vertex; 4] = [ // TriangleStrip
	Vertex { position: glam::vec3(1.0, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 1.0) },
	Vertex { position: glam::vec3(1.0, 1.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.0) },
	Vertex { position: glam::vec3(0.0, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) },
	Vertex { position: glam::vec3(0.0, 1.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) }
];

pub const ACTOR_TRIANGLE_VERT: [Vertex; 3] = [
	Vertex { position: glam::vec3(0.5, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.5) },
	Vertex { position: glam::vec3(-0.5, 0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) },
	Vertex { position: glam::vec3(-0.5, -0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) }
];
