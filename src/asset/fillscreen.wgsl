@vertex
fn main(@builtin(vertex_index) vert_idx: u32) -> @builtin(position) vec4<f32> {
	var vertices: array<vec3<f32>, 4> = array<vec3<f32>, 4> (
		vec3<f32>(-1.0, 1.0, 0.0),
		vec3<f32>(-1.0, -1.0, 0.0),
		vec3<f32>(1.0, 1.0, 0.0),
		vec3<f32>(1.0, -1.0, 0.0)
	);
	return vec4<f32>(vertices[vert_idx], 1.0);
}