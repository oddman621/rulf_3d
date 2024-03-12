@group(0) @binding(0)
var<uniform> vp: mat4x4<f32>;

struct VertexInput
{
	@location(0) position: vec3<f32>,
}

struct InstanceInput
{
	@location(1) translation0: vec4<f32>,
	@location(2) translation1: vec4<f32>,
	@location(3) translation2: vec4<f32>,
	@location(4) translation3: vec4<f32>,
	@location(5) color: vec4<f32>,
}

struct VertexOutput 
{
	@builtin(position) clip_position: vec4<f32>,
	@location(0) vert_color: vec4<f32>
}

@vertex
fn vs_main(
	vertex: VertexInput,
	instance: InstanceInput,
) -> VertexOutput 
{
	var out: VertexOutput;
	var model: mat4x4<f32> = mat4x4<f32>(instance.translation0, instance.translation1, instance.translation2, instance.translation3);
	out.clip_position = vp * model * vec4<f32>(vertex.position, 1.0);
	out.vert_color = instance.color;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> 
{
	return in.vert_color;
}