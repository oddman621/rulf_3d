/*
DDA 알고리즘 사용. 맵의 그리드 길이가 1.0으로 일정한 정사각형.

ray의 각도: a
ray의 direction: (cos_a, sin_a)

플레이어의 위치: Vec2

위치를 grid_size로 나눠 tilemap 좌표공간에 매핑

어떤 벽?

*/

pub mod wall;
pub mod floor;