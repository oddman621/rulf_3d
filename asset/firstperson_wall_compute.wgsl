
struct SurfaceInfo {
	width: u32,
	height: u32
}

struct CameraInfo {
	tilepos: vec2<f32>,
	dirvec: vec2<f32>,
	plane: vec2<f32>,
	near: f32,
	far: f32
}

struct TileMapInfo {
	size: vec2<u32>, // x=width, y=height. Also used for out_of_bound.
	tile_texids: array<i32> // if texid!=-1(= if tile has texture), this tile is solid(wall).
}

struct RaycastData {
	distance: f32,
	depth: f32,
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

// Get vector of ray by gid and do single raycasting per compute unit.
@compute @workgroup_size(1)
fn multiraycast(@builtin(global_invocation_id) gid: vec3<u32>) {
	let rayvec = camera.dirvec + camera.plane * (0.5 - f32(gid.x) / f32(raydata.raycount));
	raydata.data[gid.x] = raycast(rayvec);
}

fn raycast(rayvec: vec2<f32>) -> RaycastData {

	// Initialize variables for while loop.

	let delta_dist = 1.0 / abs(rayvec);
	let step = sign(rayvec);

	var side_dist: vec2<f32>;

	// rayvec: Left or Right?
	if rayvec.x < 0.0 {
		side_dist.x = fract(camera.tilepos.x) * delta_dist.x;
	} else {
		side_dist.x = (1.0 - fract(camera.tilepos.x)) * delta_dist.x;
	}

	// rayvec: Up or Down?
	if rayvec.y < 0.0 {
		side_dist.y = fract(camera.tilepos.y) * delta_dist.y;
	} else {
		side_dist.y = (1.0 - fract(camera.tilepos.y)) * delta_dist.y;
	}


	var tile_coord = vec2<i32>(camera.tilepos);
	var side = 0;

	// While the ray is not out of bound(The ray is not out of edge of the map)...
	while !out_of_bound(tile_coord) {

		// March ray until reaching another tile.
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
		if texid != -1 { // If the tile is solid
			var result: RaycastData;
			result.texid = texid;
			switch side {
				case 0: { // x axis
					result.distance = side_dist.x - delta_dist.x;
					let point_of_collision = camera.tilepos + rayvec * result.distance;
					let frc = fract(point_of_collision).y;
					if rayvec.x > 0.0 {
						result.u_offset = frc;
					} else {
						result.u_offset = 1.0 - frc;
					}
				}
				case 1: { // y axis
					result.distance = side_dist.y - delta_dist.y;
					let point_of_collision = camera.tilepos + rayvec * result.distance;
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
			result.depth = (result.distance - camera.near) / (camera.far - camera.near);
			return result;
		}
	} // Loop end means the raycasting is failure.

	return RaycastData(0.0, 1.0, -1, 0.0); // Return default.
}

// Helper function for readability.
// Check tilepos value is out of the tilemap.
fn out_of_bound(tilepos:vec2<i32>) -> bool {
	if tilepos.x <= 0 || tilepos.y <= 0 || tilepos.x >= i32(tilemap.size.x) || tilepos.y >= i32(tilemap.size.y) {
		return true;
	} else {
		return false;
	}
}