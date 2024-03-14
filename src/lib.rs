use std::{sync::Arc, time::{Duration, Instant}};
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/*

lib.rs의 역할? ...

이 라이브러리를 사용하는 다른 크레이트와의 외부소통창구

 - 창 띄우기
 - 프로그램 실행
 - 게임 돌리기

 윈도우는 여기서 관리됨(+입력)



File: 각종 파일 저장 로드

Game: 게임 및 로직

Input: 입력 State 저장

Rendering: 화면 그리기. 게임장면+GUI. WebGPU, OpenGL, ...

 */


mod rendering;
mod game;
mod input;

pub struct Rulf3D {
	event_loop: EventLoop<()>,
	window: Arc<Window>,
	webgpu: rendering::WebGPU,
	// separate engine and data
}


impl Rulf3D {
	pub fn new() -> Self {
		let event_loop = EventLoop::new().unwrap();
		let window = Arc::new(Window::new(&event_loop).unwrap());
		let webgpu = rendering::WebGPU::new(window.clone());

		Self { event_loop, window, webgpu }
	}

	pub fn testrun() -> Result<(), winit::error::EventLoopError> {
		let mut rulf3d = Self::new();
		let mut input_state = input::InputState::default();
		let mut minimap_renderer = rendering::minimap::Renderer::new(&rulf3d.webgpu);
		let mut game_world = game::GameWorld::test_gameworld();

		let process_tickrate = Duration::from_secs_f64(60.0f64.recip());
        let mut last_process_tick = Instant::now();
        let mut last_mouse_pos = glam::Vec2::default();
        rulf3d.event_loop.run(
            move |event, elwt| 
            match event 
            {
                Event::WindowEvent { event, window_id } if window_id == rulf3d.window.id() => 
                match event 
                {
                    WindowEvent::KeyboardInput { event: winit::event::KeyEvent { 
						physical_key: winit::keyboard::PhysicalKey::Code(keycode), 
						state, repeat: false, .. 
					}, .. } => input_state.set_key_state(keycode, state.is_pressed()),
                    WindowEvent::MouseInput { state, button, .. } => {
						match button {
							winit::event::MouseButton::Left => input_state.set_mouse_left_pressed(state.is_pressed()),
							winit::event::MouseButton::Right => input_state.set_mouse_right_pressed(state.is_pressed()),
							_ => ()
						};
					},
                    WindowEvent::CursorMoved { position, .. } => {
                        let pos = glam::vec2(position.x as f32, position.y as f32);
                        let rel = pos - last_mouse_pos;
                        last_mouse_pos = pos;
                        input_state.set_mouse_x_relative(rel.x);
                    },
                    WindowEvent::RedrawRequested => {
						let cam_pos = glam::Mat4::from_translation(game_world.get_player_position().extend(0.0));
						let cam_rot = glam::Mat4::IDENTITY;//glam::Mat4::from_rotation_z(-std::f32::consts::FRAC_PI_2 + self.scene.get_player_angle());
						let view = cam_rot.inverse() * cam_pos.inverse();
						let proj = glam::Mat4::orthographic_lh(-400.0, 400.0, -300.0, 300.0, -0.001, 1.0001);
						let viewproj = proj * view;
						
						// for wall rendering
						let wall_offsets: Vec<glam::UVec2> = game_world.walls_offset().into_iter().collect();
						let gridsize = game_world.tile_grid_size();

						// for actors rendering
						let actors_pos = game_world.actors_position();
						let actors_angle = game_world.actors_angle();
						let actor_color = glam::vec4(0.3, 0.2, 0.1, 1.0);
						minimap_renderer.draw(&rulf3d.webgpu,
							&wgpu::Color{r:0.1, g:0.2, b:0.3, a:1.0}, &viewproj, wall_offsets.as_slice(), 
							&gridsize, actors_pos.as_slice(), actors_angle.as_slice(), &actor_color);
					},
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) if physical_size.width > 0 && physical_size.height > 0 
					=> rulf3d.webgpu.reconfigure_surface_size(physical_size.width, physical_size.height),
                    _ => ()
                },
                // Event::NewEvents(StartCause::Init) =>{
                //     // NOTE: startup things
                // }
                Event::NewEvents(StartCause::Poll | StartCause::ResumeTimeReached { .. } | StartCause::WaitCancelled { .. }) =>
                {
                    let last_process_time = Instant::now().duration_since(last_process_tick);
                    if last_process_time >= process_tickrate {
                        let delta = last_process_time.as_secs_f64();
                        last_process_tick = Instant::now(); // doing process point
                        
                        let dir_input_vec = input_state.get_dir_input_vector();
						let wishdir = game_world.get_player_forward_vector().rotate((-glam::Vec2::Y).rotate(dir_input_vec));
						game_world.translate_player(wishdir * 100.0 * delta as f32);

						let mouse_rel_x = input_state.take_mouse_x_relative();
						game_world.rotate_player(-mouse_rel_x.to_radians() * 100.0 * delta as f32);

                        rulf3d.window.request_redraw();
                    }
                },
                Event::AboutToWait =>
                {
                    let last_process_time = Instant::now().duration_since(last_process_tick);
                    if last_process_time >= process_tickrate {
                        elwt.set_control_flow(ControlFlow::Poll);
                    }
                    else {
                        elwt.set_control_flow(ControlFlow::WaitUntil(Instant::now() + process_tickrate.mul_f64(0.6) - last_process_time)); 
                    }
                },
                _ => ()
            }
        )
	}

	// pub fn run(&self, game_data: GameData) {
	// 	todo!()
	// }
}



