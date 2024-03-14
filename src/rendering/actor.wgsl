@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> size: f32;

struct VertexInput {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
	@location(2) uv: vec2<f32>
}

struct InstanceInput {
	@location(3) position: vec2<f32>,
	@location(4) angle: f32 //radian
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec3<f32>
}

@vertex
fn vs_main(in_vert: VertexInput, in_inst: InstanceInput) -> VertexOutput {
	var out: VertexOutput;
	var rotated_vertex = vec2<f32>( in_vert.position.x * cos(in_inst.angle) - in_vert.position.y * sin(in_inst.angle),
									in_vert.position.y * cos(in_inst.angle) + in_vert.position.x * sin(in_inst.angle) );
	var pos = rotated_vertex * size + in_inst.position;

	out.clip_position = view_proj * vec4<f32>(pos, in_vert.position.z, 1.0);
	out.color = in_vert.color;
	return out;
}

@group(0) @binding(2)
var<uniform> color: vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	//return vec4<f32>(color.rgb + in.color, color.a);
	return color;
}