@group(0) @binding(0)
var<uniform> mvp: mat4x4<f32>;

struct VertexInput
{
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
	@location(2) uv: vec2<f32>
}

struct InstanceInput
{
	@location(3) wall_offset: vec2<u32>
}

struct VertexOutput 
{
	@builtin(position) clip_position: vec4<f32>,
	@location(0) vert_color: vec3<f32>,
	@location(1) uv: vec2<f32>
}

@vertex
fn vs_main(
	vertex: VertexInput,
	instance: InstanceInput,
) -> VertexOutput 
{
	var out: VertexOutput;
	out.clip_position = mvp * vec4<f32>(vertex.position.x + f32(instance.wall_offset.x), vertex.position.y - f32(instance.wall_offset.y), vertex.position.z, 1.0);
	out.vert_color = vertex.color;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> 
{
	return vec4<f32>(in.vert_color, 1.0);
}