struct SurfaceInfo {
	width: u32,
	half_height: u32,
	scale: u32 // TODO: No use. Delete it or use it.
};

struct RaycastData {
	distance: f32,
	texid: u32,
	u_offset: f32
};

struct RaycastDataArray {
	raycount: u32,
	data: array<RaycastData>
}

@group(0) @binding(0) var<uniform> surface_info: SurfaceInfo;
@group(0) @binding(1) var<storage, read> raycast_data_array: RaycastDataArray;

@group(1) @binding(0) var wall_texture_array: texture_2d_array<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

struct FragmentOutput {
	@location(0) color: vec4<f32>,
	@builtin(frag_depth) depth: f32
}

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> FragmentOutput {

	var x_ratio = pos.x / f32(surface_info.width);
	var raycount = raycast_data_array.raycount;
	var index = u32(f32(i32(raycount) - 1) * x_ratio);

	var distance = raycast_data_array.data[index].distance;

	var wall_height_ratio = 250.0 / distance;
	var wall_half_height = f32(surface_info.half_height) * wall_height_ratio;

	var wall_min = f32(surface_info.half_height) - wall_half_height;
	var wall_max = f32(surface_info.half_height) + wall_half_height;

	var u = raycast_data_array.data[index].u_offset;
	var v = (pos.y - wall_min) / (wall_max - wall_min);
	var uv = vec2<f32>(u, v);
	var layer = raycast_data_array.data[index].texid;

 	var color = textureSample(wall_texture_array, texture_sampler, uv, layer);

	// https://github.com/gfx-rs/naga/issues/1218#issuecomment-900499045
	// [tl;dr] discard should be after textureSample because of
	// non uniform control flow error (discard) with textureSample
	if abs(f32(surface_info.half_height) - pos.y) > wall_half_height {
	 	discard;
	}

	var out: FragmentOutput;
	out.color = color;
	out.depth = 1.0;
	
	return out;
}

