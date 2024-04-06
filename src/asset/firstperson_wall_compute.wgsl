
struct SurfaceInfo {
	width: u32,
	height: u32
}

struct CameraInfo {
	tiledpos: vec2<f32>,
	dirvec: vec2<f32>,
	plane: vec2<f32>
}

struct TileMapInfo {
	size: vec2<u32>,
	tile_texids: array<i32>
}

struct RaycastData {
	distance: f32,
	texid: i32,
	u_offset: f32
}

struct RaycastDataArray {
	raycount: u32,
	data: array<RaycastData>
}

@group(0) @binding(0) var<uniform> surface: SurfaceInfo;
@group(0) @binding(1) var<uniform> camera: CameraInfo;
@group(0) @binding(2) var<storage, read> tilemap: TileMapInfo;

@group(0) @binding(3) var<storage, read_write> raydata: RaycastDataArray;

@compute @workgroup_size(1)
fn multiraycast(@builtin(global_invocation_id) gid: vec3<u32>) {
	let rayvec = camera.dirvec + camera.plane * (0.5 - f32(gid.x) / f32(raydata.raycount));
	raydata.data[gid.x] = raycast(rayvec);
}

fn raycast(rayvec: vec2<f32>) -> RaycastData {
	let delta_dist = 1.0 / abs(rayvec);
	let step = sign(rayvec);

	var side_dist: vec2<f32>;
	if rayvec.x < 0.0 {
		side_dist.x = fract(camera.tiledpos.x) * delta_dist.x;
	} else {
		side_dist.x = (1.0 - fract(camera.tiledpos.x)) * delta_dist.x;
	}
	if rayvec.y < 0.0 {
		side_dist.y = fract(camera.tiledpos.y) * delta_dist.y;
	} else {
		side_dist.y = (1.0 - fract(camera.tiledpos.y)) * delta_dist.y;
	}
	var tile_coord = vec2<i32>(camera.tiledpos);
	var side = 0;

	while !out_of_bound(tile_coord) {
		if side_dist.x < side_dist.y {
			side_dist.x += delta_dist.x;
			tile_coord.x += i32(step.x);
			side = 0;
		} else {
			side_dist.y += delta_dist.y;
			tile_coord.y += i32(step.y);
			side = 1;
		}

		let i = u32(tile_coord.y * i32(tilemap.size.x) + tile_coord.x);
		let texid = tilemap.tile_texids[i];
		if texid != -1 {
			var result: RaycastData;
			result.texid = texid;
			switch side {
				case 0: {
					result.distance = side_dist.x - delta_dist.x;
					let point_of_collision = camera.tiledpos + rayvec * result.distance;
					let frc = fract(point_of_collision).y;
					if rayvec.x > 0.0 {
						result.u_offset = frc;
					} else {
						result.u_offset = 1.0 - frc;
					}
				}
				case 1: {
					result.distance = side_dist.y - delta_dist.y;
					let point_of_collision = camera.tiledpos + rayvec * result.distance;
					let frc = fract(point_of_collision).x;
					if rayvec.y < 0.0 {
						result.u_offset = frc;
					} else {
						result.u_offset = 1.0 - frc;
					}
				}
				default: {
					break;
				}
			}
			return result;
		}
	}

	var defval: RaycastData;
	defval.distance = 0.0;
	defval.texid = -1;
	defval.u_offset = 0.0;
	return defval;
}

fn out_of_bound(tilepos:vec2<i32>) -> bool {
	if tilepos.x <= 0 || tilepos.y <= 0 || tilepos.x >= i32(tilemap.size.x) || tilepos.y >= i32(tilemap.size.y) {
		return true;
	} else {
		return false;
	}
}