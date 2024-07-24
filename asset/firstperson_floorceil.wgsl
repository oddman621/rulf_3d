// BUG: Some parts of floors aren't drawn when resolution is about 4K or higher.

struct SurfaceInfo {
	width: u32,
	height: u32
}

struct CameraInfo {
	pos: vec2<f32>,
	pos_z: f32, // Camera height
	len: f32,
	leftmost_ray: vec2<f32>,
	rightmost_ray: vec2<f32>,
	near: f32,
	far: f32
}

struct TileMapInfo {
	size: vec2<u32>,
	tile_texids: array<vec2<i32>>
}

struct ScanlineData {
	depth: f32,
	floor: vec2<f32>,
	floor_step: vec2<f32>
}

struct PixelInfo {
	texid: vec2<i32>,
	texuv: vec2<f32>
}

@group(0) @binding(0) var<uniform> surface: SurfaceInfo;
@group(0) @binding(1) var<uniform> camera: CameraInfo;
@group(0) @binding(2) var<storage, read> tilemap: TileMapInfo;

@group(1) @binding(0) var<storage, read_write> scanlines: array<ScanlineData>; // intermediate result
@group(1) @binding(1) var<storage, read_write> pixels: array<PixelInfo>; // final result

@group(2) @binding(0) var floor_texture_array: texture_2d_array<f32>;
@group(2) @binding(1) var ceil_texture_array: texture_2d_array<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@compute @workgroup_size(1)
fn scanline_process(
	@builtin(global_invocation_id) gid: vec3<u32>
) {
	let n = gid.x;
	let p = f32(n) - f32(surface.height) / 2.0;
	let row_distance = abs(camera.pos_z * camera.len / p);

	var scanline: ScanlineData;

	let cam_plane = (camera.rightmost_ray - camera.leftmost_ray);
	scanline.floor_step = row_distance * cam_plane / f32(surface.width);
	scanline.floor = camera.pos + row_distance * camera.leftmost_ray;
	scanline.depth = (row_distance * 2.0 - camera.near) / (camera.far - camera.near); // HACK: Coincidence Workaround Side Effect. Remultiply magic number here.

	scanlines[u32(n)] = scanline;
}

// NOTE: No use because of high gpu usage.
// pixproc is done in fs_main(fragment).
// Do not use this.
@compute @workgroup_size(1)
fn pixel_process(
	@builtin(global_invocation_id) gid: vec3<u32>
) {
	//pixproc(gid.x, gid.y);
}

fn pixproc(w: u32, h: u32) {
	let i = h * surface.width + w;
	let coord = scanlines[h].floor + scanlines[h].floor_step * f32(w);

	pixels[i].texuv = fract(coord);

	let tile_coord = vec2<u32>(u32(coord.x), u32(coord.y));
	if tile_coord.x > tilemap.size.x || tile_coord.y > tilemap.size.y {
		pixels[i].texid = vec2<i32>(-1, -1);
	}
	else {
		pixels[i].texid = vec2<i32>(tilemap.tile_texids[tile_coord.y * tilemap.size.x + tile_coord.x]);
	}
}

struct FragmentOutput {
	@location(0) color: vec4<f32>,
	@builtin(frag_depth) depth: f32
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> FragmentOutput {
	pixproc(u32(pos.x), u32(pos.y));
	let is_floor = pos.y - f32(surface.height) / 2.0 > 0.0;
	let i = u32(pos.y) * surface.width + u32(pos.x);
	let uv = pixels[i].texuv;
	let ceil_texid = pixels[i].texid[0];
	let floor_texid = pixels[i].texid[1];

	let color_ceil = textureSample(ceil_texture_array, texture_sampler, uv, u32(ceil_texid));
	let color_floor = textureSample(floor_texture_array, texture_sampler, uv, u32(floor_texid));

	var out: FragmentOutput;

	if is_floor {
		if floor_texid < 0 {
			discard;
		} else {
			out.color = color_floor;
		}
	} else {
		if ceil_texid < 0 {
			discard;
		} else {
			out.color = color_ceil;
		}
	}

	out.depth = scanlines[u32(pos.y)].depth;
	return out;
}