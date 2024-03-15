@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> grid_size: vec2<f32>;

struct VertexInput {
	@location(0) position: vec3<f32>,
	@location(1) color: vec3<f32>,
	@location(2) uv: vec2<f32>
}

struct InstanceInput {
	@location(3) pos_offset: vec2<u32>,
	@location(4) texid: u32 // !
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec3<f32>,
	@location(1) uv: vec2<f32>,
	@location(2) layer: u32 // !
}

@vertex
fn vs_main(
	in_vert: VertexInput, in_inst: InstanceInput
) -> VertexOutput {
	var out: VertexOutput;
	var offset = vec2<f32>(grid_size * vec2<f32>(in_inst.pos_offset));
	var scaled_vertex = in_vert.position.xy * grid_size;
	out.clip_position = view_proj * vec4<f32>(scaled_vertex.xy + offset, in_vert.position.z, 1.0);
	out.color = in_vert.color;
	out.uv = in_vert.uv;
	out.layer = in_inst.texid;
	return out;
}

@group(0) @binding(2)
//var texture: texture_2d<f32>; //TODO: Get textures as array for verieties of walls.
var texture_array: texture_2d_array<f32>; // !

@group(0) @binding(3)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	var tex_color = textureSample(texture_array, texture_sampler, in.uv, in.layer);
	return vec4<f32>(tex_color.rgb * in.color, tex_color.a);
}