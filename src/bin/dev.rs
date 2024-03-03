use wgpu::RenderPassDescriptor;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex
{
	position: [f32; 3],
	color: [f32; 3],
	uv: [f32; 2]
}


struct DrawMap
{
	vertex_buffer: wgpu::Buffer,
	instance_buffer: wgpu::Buffer,
	render_pipeline: wgpu::RenderPipeline,
	depth_texture: wgpu::Texture,
	mvp_buffer: wgpu::Buffer,
	bind_group: wgpu::BindGroup,
	
	rotation_angle: f64,
	input_left: bool,
	input_right: bool
}

impl DrawMap
{
	const MAP_WALLS: [[u32; 2]; 63] = [ 
		[0,0], [1,0], [2,0], [3,0], [4,0], [5,0], [6,0], [7,0], [8,0], [9,0], [10,0], [11,0], [12,0], 
		[0,1],																				  [12,1],
		[0,2],																				  [12,2],
		[0,3],				 [3,3], [4,3], [5,3],				[8,3], [9,3], [10,3],		  [12,3],
		[0,4],							   [5,4],							  [10,4],		  [12,4],
		[0,5],							   [5,5],							  [10,5],		  [12,5],
		[0,6],							   [5,6],							  [10,6],		  [12,6],
		[0,7],			     [3,7], [4,7], [5,7],											  [12,7],
		[0,8],																				  [12,8],
		[0,9],																				  [12,9],
		[0,10],									  [6,10],		[8,10],						  [12,10],
		[0,11],[1,11],[2,11], [3,11], [4,11],[5,11],[6,11],[7,11],[8,11],[9,11],[10,11],[11,11],[12,11], 

	];
	const QUAD_VERTICES: [Vertex; 4] = [
		Vertex { position: [0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
		Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
		Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
		Vertex { position: [-0.5, 0.5, 0.0], color: [0.0, 1.0, 1.0], uv: [0.0, 0.0] },
	];
	const QUAD_SHADER_SOURCE: &'static str = include_str!("shader/quad_instance.wgsl");

}

impl rulf_3d::DevLoop for DrawMap
{
    fn init(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self
	{
		let vertex_buffer = device.create_buffer(
			&wgpu::BufferDescriptor 
			{ 
				label: Some("DrawMap Quad Vertex Buffer"), 
				size: std::mem::size_of_val(&DrawMap::QUAD_VERTICES) as u64, 
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
				mapped_at_creation: false 
			}
		);
		queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&DrawMap::QUAD_VERTICES));
		let instance_buffer = device.create_buffer(
			&wgpu::BufferDescriptor
			{
				label: Some("DrawMap Quad Instance Buffer"),
				size: std::mem::size_of_val(&DrawMap::MAP_WALLS) as u64,
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false
			}
		);
		queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(&DrawMap::MAP_WALLS));
		let mvp_buffer = device.create_buffer(
			&wgpu::BufferDescriptor
			{
				label: Some("DrawMap MVP Buffer"),
				size: std::mem::size_of::<glam::Mat4>() as u64,
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false
			}
		);
		let shader = device.create_shader_module(
			wgpu::ShaderModuleDescriptor
			{
				label: Some("DrawMap Quad Instance Shader Module"),
				source: wgpu::ShaderSource::Wgsl(DrawMap::QUAD_SHADER_SOURCE.into())
			}
		);
		let depth_texture = device.create_texture(
			&wgpu::TextureDescriptor
			{
				label: Some("DrawMap Depth Texture"),
				mip_level_count: 1,
				sample_count: 1,
				size: wgpu::Extent3d
				{
					width: 1, height: 1, depth_or_array_layers: 1
				},
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Depth32Float,
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
				view_formats: &[]
			}
		);

		let bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor
			{
				label: Some("DrawMap Bind Group Layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry
					{
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer
						{
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					}
				]
			}
		);
		let bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor
			{
				label: Some("DrawMap Bind Group"),
				layout: &bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry
					{
						binding: 0,
						resource: mvp_buffer.as_entire_binding()
					}
				]
			}
		);

		let pipeline_layout = device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor
			{
				label: Some("Drawmap Pipeline Layout"),
				bind_group_layouts: &[&bind_group_layout],
				push_constant_ranges: &[]
			}
		);
		let render_pipeline = device.create_render_pipeline(
			&wgpu::RenderPipelineDescriptor
			{
				label: Some("DrawMap Render Pipeline"),
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState
				{
					module: &shader,
					entry_point: "vs_main",
					buffers: &[
						wgpu::VertexBufferLayout
						{
							array_stride: std::mem::size_of::<Vertex>() as u64,
							step_mode: wgpu::VertexStepMode::Vertex,
							attributes: &[
								wgpu::VertexAttribute
								{
									format: wgpu::VertexFormat::Float32x3,
									offset: 0,
									shader_location: 0
								},
								wgpu::VertexAttribute
								{
									format: wgpu::VertexFormat::Float32x3,
									offset: std::mem::size_of::<[f32;3]>() as u64,
									shader_location: 1
								},
								wgpu::VertexAttribute
								{
									format: wgpu::VertexFormat::Float32x2,
									offset: std::mem::size_of::<[f32;3+3]>() as u64,
									shader_location: 2
								}
							]
						},
						wgpu::VertexBufferLayout
						{
							array_stride: std::mem::size_of::<[u32;2]>() as u64,
							step_mode: wgpu::VertexStepMode::Instance,
							attributes: &[
								wgpu::VertexAttribute
								{
									format: wgpu::VertexFormat::Uint32x2,
									offset: 0,
									shader_location: 3
								}
							]
						}
					]
				},
				primitive: wgpu::PrimitiveState
				{
					topology: wgpu::PrimitiveTopology::TriangleStrip,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back),
					unclipped_depth: false,
					polygon_mode: wgpu::PolygonMode::Fill,
					conservative: false
				},
				depth_stencil: Some(wgpu::DepthStencilState
				{
					format: wgpu::TextureFormat::Depth32Float,
					depth_write_enabled: true,
					depth_compare: wgpu::CompareFunction::Less,
					stencil: wgpu::StencilState::default(),
					bias: wgpu::DepthBiasState::default()
				}),
				multisample: wgpu::MultisampleState::default(),
				fragment: Some(wgpu::FragmentState
				{
					module: &shader,
					entry_point: "fs_main",
					targets: &[
						Some(wgpu::ColorTargetState
							{
								format: surface_format,
								blend: Some(wgpu::BlendState
									{
										color: wgpu::BlendComponent::REPLACE,
										alpha: wgpu::BlendComponent::REPLACE
									}
								),
								write_mask: wgpu::ColorWrites::ALL
							}
						)
					]
				}),
				multiview: None
			}
		);

		Self
		{
		vertex_buffer,
		instance_buffer,
		render_pipeline,
		depth_texture,
		mvp_buffer,
		bind_group,
		rotation_angle: f64::default(),
		input_left: false,
		input_right: false
		}
	}
    fn process(&mut self, delta: f64)
	{
		if self.input_left
		{
			self.rotation_angle -= 15.0 * delta;
		}
		if self.input_right
		{
			self.rotation_angle += 15.0 * delta;
		}
	}
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface)
	{
		let output = surface.get_current_texture().unwrap();
		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		if self.depth_texture.size() != size
		{
			self.depth_texture.destroy();
			self.depth_texture = device.create_texture(
				&wgpu::TextureDescriptor
				{
					label: Some("DrawMap Depth Texture"),
					size: wgpu::Extent3d
					{
						width: size.width,
						height: size.height,
						depth_or_array_layers: 1
					},
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Depth32Float,
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
					view_formats: &[]
				}
			);
		}
		let depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

		const CAMERA_WIDTH: f32 = 18.0;
		const CAMERA_HEIGHT: f32 = 13.5;
		let model = glam::Mat4::from_translation(glam::vec3(0.5, -0.5, 0.0));
		let cam_pos = glam::vec3(0.0, 0.0, -0.1);
		let cam_rot = glam::Quat::from_rotation_z(self.rotation_angle.to_radians() as f32);
		let cam_view = glam::Mat4::from_rotation_translation(-cam_rot, -cam_pos);
		let ortho = glam::Mat4::orthographic_lh(0.0, CAMERA_WIDTH, -CAMERA_HEIGHT, 0.0, 0.1, 100.0);
		let mvp = ortho * cam_view * model;
		queue.write_buffer(&self.mvp_buffer, 0, bytemuck::cast_slice(mvp.as_ref()));
		//queue.write_buffer(&self.mvp_buffer, 0, bytemuck::cast_slice(glam::Mat4::IDENTITY.as_ref()));

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
		let mut render_pass = encoder.begin_render_pass(
			&RenderPassDescriptor
			{
				label: Some("DrawMap Render Pass"),
				color_attachments: &[
					Some(wgpu::RenderPassColorAttachment 
					{ 
						view: &view, 
						resolve_target: None, 
						ops: wgpu::Operations
						{
							load: wgpu::LoadOp::Clear(wgpu::Color{r: 0.1, g: 0.2, b: 0.3, a: 1.0}),
							store: wgpu::StoreOp::Store
						}
					})
				],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment
				{
					view: &depth_texture_view,
					depth_ops: Some(wgpu::Operations{
						load: wgpu::LoadOp::Clear(1.0),
						store: wgpu::StoreOp::Store
					}),
					stencil_ops: None
				}),
				..Default::default()
			}
		);

		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_bind_group(0, &self.bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
		render_pass.draw(0..4, 0..DrawMap::MAP_WALLS.len() as u32);
		drop(render_pass);

		queue.submit(Some(encoder.finish()));
		output.present();
	}
}

impl rulf_3d::InputEvent for DrawMap
{
	fn keyboard_input(&mut self, keycode: winit::keyboard::KeyCode, state: winit::event::ElementState) 
	{
		use winit::keyboard::KeyCode;

		match keycode
		{
			KeyCode::ArrowLeft => self.input_left = state.is_pressed(),
			KeyCode::ArrowRight => self.input_right = state.is_pressed(),
			_ => ()
		}
	}
}

fn main()
{
	rulf_3d::run_dev::<DrawMap>();
}