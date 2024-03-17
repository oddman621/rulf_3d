@group(0) @binding(0)
var<uniform> viewproj: mat4x4<f32>;

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec3<f32>
}

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) color: vec3<f32>) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = viewproj * vec4<f32>(position, 1.0);
	out.color = color;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(in.color, 1.0);
}