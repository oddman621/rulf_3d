use wgpu::util::DeviceExt;
use crate:: {
	webgpu::{WebGPU, WebGPUDevice, WebGPUConfig},
	asset::AssetServer,
	geometry::{Vertex, ACTOR_TRIANGLE_VERT}
};

pub struct ActorRender {

	pub vb: wgpu::Buffer,
	pub instb: wgpu::Buffer,
	pub instb_len: u32,
	pub viewproj_ub: wgpu::Buffer,
	pub color_ub: wgpu::Buffer,
	pub actorsize_ub: wgpu::Buffer,

	pub bind_group: wgpu::BindGroup,
	pub pipeline: wgpu::RenderPipeline
}

impl ActorRender {
	const MAX_ACTOR_INSTANCE: u64 = 512;
	pub fn new(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
		let (device, _) = webgpu.get_device();
		let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("ActorRender::vb"),
			usage: wgpu::BufferUsages::VERTEX,
			contents: bytemuck::cast_slice(&ACTOR_TRIANGLE_VERT)
		});

		let instb = device.create_buffer(&wgpu::BufferDescriptor { //pos: [f32;2], ang: f32
			label: Some("ActorRender::instb"),
			size: std::mem::size_of::<f32>() as u64 * 3 * Self::MAX_ACTOR_INSTANCE,
			usage:wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let viewproj_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ActorRender::viewproj_ub"),
			size: std::mem::size_of::<glam::Mat4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		
		let actorsize_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ActorRender::actorsize_ub"),
			size: std::mem::size_of::<f32>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let color_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ActorRender::color_ub"),
			size: std::mem::size_of::<glam::Vec4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("ActorRender bind group layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry { //view projection mat4x4
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer { 
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry { //size
					binding: 1,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry { //color
					binding: 2,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				}
			]
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("ActorRender::bind_group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: viewproj_ub.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: actorsize_ub.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: color_ub.as_entire_binding()
				}
			]
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("ActorRender pipeline layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[]
		});
		let shader_module = asset_server.get_shader("minimap_actor").unwrap();

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
			label: Some("ActorRender::render_pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: shader_module,
				entry_point: "vs_main",
				buffers: &[
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as u64,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &Vertex::VERT_ATTR
					},
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<f32>() as u64 * 3,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x2,
								offset: 0,
								shader_location: 3
							},
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32,
								offset: std::mem::size_of::<f32>() as u64 * 2,
								shader_location: 4
							}
						]
					}
				]
			},
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				unclipped_depth: false,
				polygon_mode: wgpu::PolygonMode::Fill,
				conservative: false
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: shader_module,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: webgpu.get_config().format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				})]
			}),
			multiview: None
		});

		Self {
			vb, instb, instb_len: 0, viewproj_ub, color_ub, actorsize_ub, bind_group, pipeline: render_pipeline
		}
	}
}
