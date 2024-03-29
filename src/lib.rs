use std::{sync::Arc, time::{Duration, Instant}};
use winit::{
    dpi::PhysicalPosition, event::{Event, StartCause, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::Window
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


mod webgpu;
mod game;
mod input;
mod minimap;
mod raycasting;
mod firstperson;

pub struct Rulf3D;


impl Rulf3D {
	pub fn testrun() -> Result<(), winit::error::EventLoopError> {
		let event_loop = EventLoop::new().unwrap();
		let window = Arc::new(Window::new(&event_loop).unwrap());
		let mut webgpu = webgpu::WebGPU::new(window.clone());
		let mut input_state = input::InputState::default();
		let mut minimap_renderer = minimap::Renderer::new(&webgpu);
        let mut firstperson_renderer = firstperson::Renderer::new(&webgpu);
		let mut game_world = game::GameWorld::test_gameworld();

        let mut draw_minimap = false;

		let process_tickrate = Duration::from_secs_f64(60.0f64.recip());
        let mut last_process_tick = Instant::now();

        event_loop.run(
            move |event, elwt| 
            match event 
            {
                Event::WindowEvent { event, window_id } if window_id == window.id() => 
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
                        // TODO: Turning is too stiff
                        let center = PhysicalPosition {x: window.inner_size().width / 2, y: window.inner_size().height / 2};
                        let _ = window.set_cursor_position(center);
                        let pos = glam::vec2(position.x as f32, position.y as f32);
                        let rel = pos - glam::vec2(center.x as f32, center.y as f32);
                        input_state.set_mouse_x_relative(rel.x);
                    },
                    WindowEvent::RedrawRequested => {
                        if draw_minimap {
                            minimap_renderer.render(&webgpu, &game_world, &wgpu::Color{r:0.1, g:0.2, b:0.3, a:1.0});
                        }
                        else {
                            firstperson_renderer.render(&webgpu, &game_world, &wgpu::Color{r:0.1, g:0.2, b:0.3, a:1.0});
                        }
					},
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) if physical_size.width > 0 && physical_size.height > 0 
					=> webgpu.reconfigure_surface_size(physical_size.width, physical_size.height),
                    _ => ()
                },
                Event::NewEvents(StartCause::Init) =>{
                    if let Err(e) = window.set_cursor_grab(winit::window::CursorGrabMode::Confined) {
                        println!("{:?}", e);
                    }
                }
                Event::NewEvents(StartCause::Poll | StartCause::ResumeTimeReached { .. } | StartCause::WaitCancelled { .. }) =>
                {
                    let last_process_time = Instant::now().duration_since(last_process_tick);
                    if last_process_time >= process_tickrate {
                        let delta = last_process_time.as_secs_f64();
                        last_process_tick = Instant::now();
                        
                        // input
                        let dir_input_vec = input_state.get_dir_input_vector();
						let wishdir = game_world.get_player_forward_vector().rotate((-glam::Vec2::Y).rotate(dir_input_vec));
						game_world.translate_player(wishdir * 100.0 * delta as f32);

						let mouse_rel_x = input_state.take_mouse_x_relative();
						game_world.rotate_player(-mouse_rel_x.to_radians() * 100.0 * delta as f32);

                        if input_state.is_action_just_pressed(input::Action::ToggleMinimap) {
                            draw_minimap = !draw_minimap;
                        }


                        // raycast
                        match raycasting::wall::single_raycast(
                            &game_world.get_walls(), game_world.get_grid_size(), 
                            game_world.get_player_position(), game_world.get_player_forward_vector(), 100
                        ) {
                            Some((distance, index, ..)) => window.set_title(format!("({:.1}, {})", distance, index).as_str()),
                            None => window.set_title("None")
                        }

                        window.request_redraw();
                    }
                },
                Event::AboutToWait =>
                {
                    let last_process_time = Instant::now().duration_since(last_process_tick);
                    if last_process_time >= process_tickrate {
                        elwt.set_control_flow(ControlFlow::Poll);
                    }
                    else {
                        elwt.set_control_flow(ControlFlow::WaitUntil(Instant::now() + process_tickrate.mul_f64(0.5) - last_process_time)); 
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



