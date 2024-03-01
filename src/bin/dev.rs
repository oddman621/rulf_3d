

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex
{
	position: [f32; 3],
	color: [f32; 3]
}


#[derive(Default)]
struct DrawQuad
{
	vertex_buffer: Option<wgpu::Buffer>,
	render_pipeline: Option<wgpu::RenderPipeline>,
	shader: Option<wgpu::ShaderModule>
}
impl rulf_3d::GameLoop for DrawQuad 
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

		self.vertex_buffer = Some(device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor
			{
				label: Some("DrawQuad Vertex Buffer"),
				usage: wgpu::BufferUsages::VERTEX,
				contents: bytemuck::cast_slice(VERTICES)
			}
		));

		//wgpu::ShaderSource::Wgsl(SOURCE_STR_SLICE.into())
		self.shader = Some(device.create_shader_module(wgpu::include_wgsl!("defaultshader.wgsl")));
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
				depth_stencil: None, // TODO: Add depth stencil!
				multisample: wgpu::MultisampleState::default(),
				multiview: None
			}
		));
	}
    fn process(&mut self, _delta: f64) {}
    fn render(&mut self, device: &wgpu::Device, surface: &wgpu::Surface, queue: &wgpu::Queue)
	{
		let output = surface.get_current_texture().expect("Failed to get current texture.");
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

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
				depth_stencil_attachment: None, // TODO: Add depth stencil!
				..Default::default()
			}
		);

		render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
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