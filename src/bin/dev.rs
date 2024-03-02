#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex
{
	position: [f32; 3],
	color: [f32; 3]
}

struct DrawMap;

#[derive(Default)]
struct DrawQuad
{
	vertex_buffer: Option<wgpu::Buffer>,
	render_pipeline: Option<wgpu::RenderPipeline>,
	shader: Option<wgpu::ShaderModule>,
	depth_texture: Option<wgpu::Texture>,
	mvp_buffer: Option<wgpu::Buffer>,
	bind_group: Option<wgpu::BindGroup>
}
impl rulf_3d::DevLoop for DrawQuad 
{
    fn startup(&mut self, device: &wgpu::Device, surface_format: wgpu::TextureFormat)
	{
		use wgpu::util::DeviceExt;
		const VERTICES: &[Vertex; 4] = &[
			Vertex { position: [1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0] },
			Vertex { position: [1.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },
			Vertex { position: [-1.0, -1.0, 0.0], color: [0.0, 0.0, 1.0] },
			Vertex { position: [-1.0, 1.0, 0.0], color: [0.0, 1.0, 1.0] },
		];
		const SHADER_SOURCE: &str = include_str!("shader/vert_color_render.wgsl");

		self.vertex_buffer = Some(device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor
			{
				label: Some("DrawQuad Vertex Buffer"),
				usage: wgpu::BufferUsages::VERTEX,
				contents: bytemuck::cast_slice(VERTICES)
			}
		));
		self.mvp_buffer = Some(device.create_buffer(
			&wgpu::BufferDescriptor
			{
				label: Some("DrawQuad MVP Buffer"),
				size: std::mem::size_of::<glam::Mat4>() as u64,
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false
			}
		));

		self.shader = Some(device.create_shader_module(
			wgpu::ShaderModuleDescriptor
			{
				label: Some("DrawQuad shader module"),
				source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into())
			}
		));
		self.render_pipeline = Some(device.create_render_pipeline(
			&wgpu::RenderPipelineDescriptor
			{
				label: Some("DrawQuad Render Pipeline"),
				layout: None,
				vertex: wgpu::VertexState
				{
					module: self.shader.as_ref().unwrap(),
					entry_point: "vs_main",
					buffers: &[
						wgpu::VertexBufferLayout
						{
							array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
							step_mode: wgpu::VertexStepMode::Vertex,
							attributes: &wgpu::vertex_attr_array![ 0 => Float32x3, 1 => Float32x3 ]
						}
					]
				},
				fragment: Some(wgpu::FragmentState
				{
					module: self.shader.as_ref().unwrap(),
					entry_point: "fs_main",
					targets: &[
						Some(wgpu::ColorTargetState
						{
							format: surface_format,
							blend: Some(wgpu::BlendState::REPLACE),
							write_mask: wgpu::ColorWrites::ALL
						})
					]
				}),
				primitive: wgpu::PrimitiveState
				{
					topology: wgpu::PrimitiveTopology::TriangleStrip,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back),
					polygon_mode: wgpu::PolygonMode::Fill,
					..Default::default()
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
				multiview: None
			}
		));

		self.bind_group = Some(device.create_bind_group(
			&wgpu::BindGroupDescriptor
			{
				label: Some("DrawQuad Bind Group"),
				layout: &self.render_pipeline.as_ref().unwrap().get_bind_group_layout(0),
				entries: &[
					wgpu::BindGroupEntry
					{
						binding: 0,
						resource: self.mvp_buffer.as_ref().unwrap().as_entire_binding()
					}
				]
			}
		));
	}
    fn process(&mut self, _delta: f64) {}
    fn render(&mut self, device: &wgpu::Device, surface: &wgpu::Surface, queue: &wgpu::Queue)
	{
		let output = surface.get_current_texture().expect("Failed to get current texture.");
		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		// destroy and create depth texture
		if self.depth_texture.is_some()
		{
			let prev_depth_texture = self.depth_texture.as_ref().unwrap();
			if size != prev_depth_texture.size()
			{
				self.depth_texture.as_ref().unwrap().destroy()
			}
		}

		if !self.depth_texture.as_ref().is_some_and(|f| f.size() == output.texture.size())
		{
			if self.depth_texture.is_some()
			{
				self.depth_texture.as_ref().unwrap().destroy();
			}
			self.depth_texture = Some(device.create_texture(
				&wgpu::TextureDescriptor
				{
					label: None,
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
			));
		}
		let depth_texture_view = self.depth_texture.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor::default());

		

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		let mut render_pass = encoder.begin_render_pass(
			&wgpu::RenderPassDescriptor
			{
				color_attachments: &[
					Some(wgpu::RenderPassColorAttachment
					{
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations
						{
							load: wgpu::LoadOp::Clear(wgpu::Color{r:0.1, g:0.2, b:0.3, a:1.0}),
							store: wgpu::StoreOp::Store
						}
					})
				],
				depth_stencil_attachment: Some(
					wgpu::RenderPassDepthStencilAttachment
					{
						view: &depth_texture_view,
						depth_ops: Some(wgpu::Operations
						{
							load: wgpu::LoadOp::Clear(1.0),
							store: wgpu::StoreOp::Store
						}),
						stencil_ops: None
					}
				),
				..Default::default()
			}
		);


		
		const ORTHO_WIDTH: f32 = 32.0;
		const ORTHO_HEIGHT: f32 = 24.0;
		let model = glam::Mat4::from_translation(glam::vec3(10.0, 10.0, 0.0));
		let view_pos = glam::vec3(ORTHO_WIDTH/2.0, ORTHO_HEIGHT/2.0, -2.0);
		let view = glam::Mat4::from_translation(-view_pos);
		let ortho = glam::Mat4::orthographic_lh(0.0, ORTHO_WIDTH, -ORTHO_HEIGHT, 0.0, 0.1, 100.0);

		queue.write_buffer(&self.mvp_buffer.as_ref().unwrap(), 0, bytemuck::cast_slice((ortho * view * model).as_ref()));

		render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
		render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
		render_pass.draw(0..4, 0..1);
		drop(render_pass);

		queue.submit(Some(encoder.finish()));
		output.present();
	}
}

fn main()
{
	rulf_3d::run_dev(DrawQuad::default());
}