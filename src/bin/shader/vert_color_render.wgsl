@group(0) @binding(0)
var<uniform> mvp: mat4x4<f32>;

struct VertexInput
{
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>
}

struct VertexOutput 
{
	@builtin(position) clip_position: vec4<f32>,
	@location(0) vert_color: vec3<f32>
}

@vertex
fn vs_main(
	@builtin(vertex_index) in_vertex_index: u32,
	vertex: VertexInput
) -> VertexOutput 
{
	var out: VertexOutput;
	out.clip_position = mvp * vec4<f32>(vertex.position, 1.0);
	out.vert_color = vertex.color;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> 
{
	return vec4<f32>(in.vert_color, 1.0);
}