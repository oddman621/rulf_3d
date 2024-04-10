
use crate::{
	webgpu::{WebGPU, WebGPUDevice, WebGPUConfig},
	game::TileType
};

use super::{SurfaceInfo, RaycastData, WallCameraInfo};

pub struct Data {
	pub surface_info_buffer: wgpu::Buffer,
	pub camera_info: wgpu::Buffer,
	pub tilemap_data: wgpu::Buffer,
	pub raycast_data_array_buffer: wgpu::Buffer,
	_texture_view: wgpu::TextureView,
	_texture_sampler: wgpu::Sampler,
	pub compute_bind_group: wgpu::BindGroup,
	pub compute_pipeline: wgpu::ComputePipeline,
	pub render_bind_groups: [wgpu::BindGroup; 2],
	pub render_pipeline: wgpu::RenderPipeline
}

impl Data {
	const MAX_RAYCOUNT: u64 = 4320; //8K
	const MAX_TILESIZE: glam::U64Vec2 = glam::u64vec2(2048, 2048);
	const TILEMAP_FIELDS_DATA_SIZE: u64 = std::mem::size_of::<u32>() as u64 * 2 + std::mem::size_of::<f32>() as u64;
}

impl Data {
	pub fn new(webgpu: &WebGPU, asset_server: &crate::asset::AssetServer) -> Self {
		let (device, _) = webgpu.get_device();
		let surface_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::surface_info_buffer"),
			size: std::mem::size_of::<SurfaceInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let camera_info = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("wall::Data.camera_info"),
			size: std::mem::size_of::<WallCameraInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let tilemap_data = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("wall::Data.tilemap_data"),
			size: std::mem::size_of::<TileType>() as u64 * Self::MAX_TILESIZE.x * Self::MAX_TILESIZE.y + Self::TILEMAP_FIELDS_DATA_SIZE,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let raycast_data_array_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::raycast_data_array_buffer"),
			size: std::mem::size_of::<u32>() as u64 +  
				std::mem::size_of::<RaycastData>() as u64 * Self::MAX_RAYCOUNT,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
		
		let texture_array = asset_server.get_texture("all_6").unwrap();
		
		let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
			dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default()
		});

		let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("wall::Data compute bind group layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 3,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: false },
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				}
			]
		});

		let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("wall::Data.compute_bind_group"),
			layout: &compute_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: surface_info_buffer.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: camera_info.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: tilemap_data.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: raycast_data_array_buffer.as_entire_binding()
				}
			]
		});

		let firstperson_wall_compute_shader = asset_server.get_shader("firstperson_wall_compute").unwrap();
		let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("wall::Data compute pipeline layout"),
			bind_group_layouts: &[&compute_bind_group_layout],
			push_constant_ranges: &[]
		});

		let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("wall::Data.compute_pipeline"),
			layout: Some(&compute_pipeline_layout),
			module: firstperson_wall_compute_shader,
			entry_point: "multiraycast"
		});


		let bind_group_0_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("WallRender bind group 0 layout: surface and raydata"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: false,
						min_binding_size: None
					},
					count: None
				}
			]
		});

		let bind_group_1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("WallRender bind group 1 layout: texture array and sampler"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture { 
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2Array,
						multisampled: false
					},
					count: None
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
					count: None
				}
			]
		}); 

		let render_bind_groups = [
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("WallRender::bind_groups[0]"),
				layout: &bind_group_0_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: surface_info_buffer.as_entire_binding()
					},
					wgpu::BindGroupEntry {
						binding: 1, 
						resource: raycast_data_array_buffer.as_entire_binding()
					}
				]
			}),
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("WallRender::bind_groups[1]"),
				layout: &bind_group_1_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&texture_array_view)
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&texture_sampler)
					}
				]
			})
		];

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("WallRender pipeline layout"),
			bind_group_layouts: &[&bind_group_0_layout, &bind_group_1_layout],
			..Default::default()
		});

		let fillscreen_shader_module = asset_server.get_shader("fillscreen").unwrap();
		let firstperson_wall_shader_frag = asset_server.get_shader("firstperson_wall_frag").unwrap();

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("WallRender::pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: fillscreen_shader_module,
				entry_point: "main",
				buffers: &[]
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
			depth_stencil: Some(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::LessEqual,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default()
			}),
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: firstperson_wall_shader_frag,
				entry_point: "main",
				targets: &[Some(wgpu::ColorTargetState {
					format: webgpu.get_config().format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				})]
			}),
			multiview: None
		});

		Self {
			surface_info_buffer, 
			camera_info, tilemap_data,
			raycast_data_array_buffer,
			_texture_view: texture_array_view,
			_texture_sampler: texture_sampler,
			render_bind_groups, render_pipeline,
			compute_bind_group, compute_pipeline
		}
	}
}