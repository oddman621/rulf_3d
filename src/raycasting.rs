/*
DDA 알고리즘 사용. 맵의 그리드 길이가 1.0으로 일정한 정사각형.

ray의 각도: a
ray의 direction: (cos_a, sin_a)

플레이어의 위치: Vec2

위치를 grid_size로 나눠 tilemap 좌표공간에 매핑

어떤 벽?

*/


use std::collections::HashMap;


// Option<(거리, 벽idx)>
// https://lodev.org/cgtutor/raycasting.html
// raydir: ray의 방향. 카메라의 방향이 아님. 항상 1의 길이가 아니며, epsilon 이상의 다양한 길이를 가짐. (더 정확히 말하면 1의 길이를 가질 필요 없음)
pub fn raycast(walls: &HashMap<glam::UVec2, u32>, gridsize: f32, from: glam::Vec2, raydir: glam::Vec2, stepnum: u32) -> Option<(f32, u32)> {
	enum Side { None, NS, EW }

	let tilespace_from = from / gridsize;
	let mut tile_pos = tilespace_from.as_uvec2();

	let delta_dist_x = raydir.x.abs().recip();
	let delta_dist_y = raydir.y.abs().recip();

	let step_x = raydir.x.signum() as i32;
	let step_y = raydir.y.signum() as i32;

	let mut side_dist_x = if raydir.x.is_sign_negative() { tilespace_from.x.fract() } else { 1.0 - tilespace_from.x.fract() } * delta_dist_x;
	let mut side_dist_y = if raydir.y.is_sign_negative() { tilespace_from.y.fract() } else { 1.0 - tilespace_from.y.fract() } * delta_dist_y;
	let mut side = Side::None;
	let mut steps = 0;

	loop {
		if side_dist_x < side_dist_y {
			if tile_pos.x == 0 { return None; }
			side_dist_x += delta_dist_x;
			tile_pos.x = (tile_pos.x as i32 + step_x) as u32;
			side = Side::EW;
		}
		else {
			if tile_pos.y == 0 { return None; }
			side_dist_y += delta_dist_y;
			tile_pos.y = (tile_pos.y as i32 + step_y) as u32;
			side = Side::NS;
		}

		if let Some(texid) = walls.get(&tile_pos) {
			return match side {
				Side::EW => Some((side_dist_x - delta_dist_x, texid.clone())),
				Side::NS => Some((side_dist_y - delta_dist_y, texid.clone())),
				Side::None => None
			};
		}
		else {
			steps += 1;
			if stepnum <= steps {
				return None;
			}
		}
	};
}
