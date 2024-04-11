use wgpu::util::DeviceExt;
use crate:: {
	webgpu::{WebGPU, WebGPUDevice, WebGPUConfig},
	asset::AssetServer,
	geometry::{Vertex, QUAD_VERT}
};

pub struct WallRender {
	pub vb: wgpu::Buffer,
	pub instb: wgpu::Buffer,
	pub instb_len: u32,
	pub viewproj_ub: wgpu::Buffer,
	pub gridsize_ub: wgpu::Buffer,
	_texture_array_view: wgpu::TextureView,
	_texture_sampler: wgpu::Sampler,
	pub bind_group: wgpu::BindGroup,
	pub pipeline: wgpu::RenderPipeline,
}


impl WallRender {
	const MAX_WALL_INSTANCE: u64 = 512 * 512;
	pub fn new(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
		let (device, _) = webgpu.get_device();
		let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("WallRender::vb"),
			contents: bytemuck::cast_slice(&QUAD_VERT),
			usage: wgpu::BufferUsages::VERTEX
		});

		let instb = device.create_buffer(&wgpu::BufferDescriptor {// offset: [u32;2], texid: u32
			label: Some("WallRender::instb"),
			size: Self::MAX_WALL_INSTANCE * std::mem::size_of::<u32>() as u64 * 3,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let viewproj_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::viewproj_ub"),
			size: std::mem::size_of::<glam::Mat4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let gridsize_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::gridsize_ub"),
			size: std::mem::size_of::<f32>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
		
		let texture_array = asset_server.get_texture("all_6_5x5").unwrap();
		
		let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
			dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default()
		});


		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("WallRender bind group layout"),
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
				wgpu::BindGroupLayoutEntry { //grid size
					binding: 1,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry { // TextureArray for Wall
					binding: 2,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2Array,
						multisampled: false
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry { // Sampler
					binding: 3,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
					count: None
				}
			]
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("WallRender::bind_group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: viewproj_ub.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: gridsize_ub.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(&texture_array_view)
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::Sampler(&texture_sampler)
				}
			]
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("WallRender pipeline layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[]
		});

		let shader_module = asset_server.get_shader("minimap_wall").unwrap();
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("WallRender::render_pipeline"),
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
						array_stride: std::mem::size_of::<[u32; 2]>() as u64 + std::mem::size_of::<u32>() as u64,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Uint32x2,
								offset: 0,
								shader_location: 3
							},
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Uint32,
								offset: std::mem::size_of::<[u32;2]>() as u64,
								shader_location: 4
							}
						]
					}
				]
			},
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleStrip,
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
			vb, instb, instb_len: 0, viewproj_ub, gridsize_ub, bind_group, pipeline: render_pipeline,
			_texture_array_view: texture_array_view, _texture_sampler: texture_sampler
		}
	}
}