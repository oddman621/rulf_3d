/*
DDA 알고리즘 사용. 맵의 그리드 길이가 1.0으로 일정한 정사각형.

ray의 각도: a
ray의 direction: (cos_a, sin_a)

플레이어의 위치: Vec2

위치를 grid_size로 나눠 tilemap 좌표공간에 매핑

어떤 벽?

*/


use std::collections::HashMap;

// pub enum MultiRaycastError {
// 	CAMFOV_IS_ABOVE_PI
// }

// pub fn multi_raycast(walls: &HashMap<glam::UVec2, u32>, gridsize: f32, from: glam::Vec2, camdir: glam::Vec2, camfov: f32, raycount: u32, stepnum: u32) -> Result<Vec<Option<(f32, u32)>>, MultiRaycastError> {
// 	let tan_half_fov = (camfov / 2.0).tan(); // cam_plane.len / camdir_len
// 	if tan_half_fov.is_infinite() {
// 		return Err(MultiRaycastError::CAMFOV_IS_ABOVE_PI);
// 	}

// 	let cam_plane = camdir.perp() * 0.5;
// 	let camdir_len = cam_plane.length() / tan_half_fov;
// 	let camdir = camdir * camdir_len;

// 	for n in 0..raycount {
// 		let rayvec = camdir + cam_plane * (0.5 - n as f32 / raycount as f32);
// 	}

// 	let raycasting: Vec<_> = (0..raycount).into_iter().map(|f| {
// 		let rayvec = camdir + cam_plane * (0.5 - f as f32 / raycount as f32);
// 		single_raycast(walls, gridsize, from, rayvec, stepnum)
// 	}).collect();

// 	Ok(raycasting)
// }

// Option<(거리, 벽idx)>
// https://lodev.org/cgtutor/raycasting.html
// rayvec: ray의 방향. 카메라의 방향이 아님. 항상 1의 길이가 아니며, epsilon 이상의 다양한 길이를 가짐.
pub fn single_raycast(walls: &HashMap<glam::UVec2, u32>, gridsize: f32, from: glam::Vec2, rayvec: glam::Vec2, stepnum: u32) -> Option<(f32, u32)> {
	

	let tilespace_from = from / gridsize;
	let mut tile_pos = tilespace_from.as_uvec2();

	let delta_dist_x = rayvec.x.abs().recip();
	let delta_dist_y = rayvec.y.abs().recip();

	let step_x = rayvec.x.signum() as i32;
	let step_y = rayvec.y.signum() as i32;

	let mut side_dist_x = if rayvec.x.is_sign_negative() { tilespace_from.x.fract() } else { 1.0 - tilespace_from.x.fract() } * delta_dist_x;
	let mut side_dist_y = if rayvec.y.is_sign_negative() { tilespace_from.y.fract() } else { 1.0 - tilespace_from.y.fract() } * delta_dist_y;
	let mut steps = 0;

	while stepnum > steps {
		enum Side { NS, EW }
		steps += 1;
		let side;
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
				Side::EW => Some(((side_dist_x - delta_dist_x) * gridsize, texid.clone())),
				Side::NS => Some(((side_dist_y - delta_dist_y) * gridsize, texid.clone()))
			};
		}
	}

	return None;
}
