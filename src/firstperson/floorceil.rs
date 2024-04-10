use crate::{
	webgpu::{WebGPU, WebGPUDevice, WebGPUConfig}, 
	asset::ShaderSource,
	asset::AssetServer
};
use super::{SurfaceInfo, FloorCeilCameraInfo, ScanlineData};



pub struct Data {
	pub surface_info: wgpu::Buffer,
	pub camera_info: wgpu::Buffer,
	pub tilemap_info: wgpu::Buffer,

	pub bind_groups: [wgpu::BindGroup; 3],
	pub compute_pipelines: [wgpu::ComputePipeline; 2],
	pub render_pipeline: wgpu::RenderPipeline,

	_scanlines: wgpu::Buffer,
	_pixels: wgpu::Buffer,

	_floor_texview: wgpu::TextureView,
	_ceil_texview: wgpu::TextureView,
	_sampler: wgpu::Sampler	
}

impl Data {
	const MAXIMUM_TILEMAP_SIZE: u64 = 2048 * 2048;
	const MAX_WIDTH: u64 = 3840;
	const MAX_HEIGHT: u64 = 2160;

	const PIXELINFO_SIZE: u64 = std::mem::size_of::<glam::IVec2>() as u64 + std::mem::size_of::<glam::Vec2>() as u64;
}

impl Data {
	pub fn new(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
		let (device, queue) = webgpu.get_device();
		let surface_info = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data.surface_info"),
			size: std::mem::size_of::<SurfaceInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let camera_info = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data.camera_info"),
			size: std::mem::size_of::<FloorCeilCameraInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let tilemap_info = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data.tilemap_info"),
			size: std::mem::size_of::<glam::UVec2>() as u64 + std::mem::size_of::<glam::IVec2>() as u64 * Self::MAXIMUM_TILEMAP_SIZE,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let scanlines = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data._scanlines"),
			size: std::mem::size_of::<ScanlineData>() as u64 * Self::MAX_HEIGHT,
			usage: wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});
		let pixels = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data._pixels"),
			size: Self::PIXELINFO_SIZE * Self::MAX_WIDTH * Self::MAX_HEIGHT,
			usage: wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});

		let tex_array = asset_server.get_texture("all_6").unwrap();
		
		let floor_texview = tex_array.create_view(&wgpu::TextureViewDescriptor {
			label: Some("floorceil::Data._floor_texview"),
			dimension: Some(wgpu::TextureViewDimension::D2Array),
			..Default::default()
		});
		let ceil_texview = tex_array.create_view(&wgpu::TextureViewDescriptor {
			label: Some("floorceil::Data._ceil_texview"),
			dimension: Some(wgpu::TextureViewDimension::D2Array),
			..Default::default()
		});

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

		let bind_group_layouts = [
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("floorceil bind group layout 0"),
				entries: & [
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer { 
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer { 
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 2,
						visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer { 
							ty: wgpu::BufferBindingType::Storage { read_only: true },
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					},
				]
			}),
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("floorceil bind group layout 1"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Storage { read_only: false },
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Storage { read_only: false },
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					}
				]
			}),
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("floorceil bind group layout 2"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
							view_dimension: wgpu::TextureViewDimension::D2Array,
							multisampled: false
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
							view_dimension: wgpu::TextureViewDimension::D2Array,
							multisampled: false
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 2,
						visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
						count: None
					}
				]
			})
		];

		let bind_groups = [
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("floorceil::Data.bind_groups[0]"),
				layout: &bind_group_layouts[0],
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: surface_info.as_entire_binding()
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: camera_info.as_entire_binding()
					},
					wgpu::BindGroupEntry {
						binding: 2,
						resource: tilemap_info.as_entire_binding()
					}
				]
			}),
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("floorceil::Data.bind_groups[1]"),
				layout: &bind_group_layouts[1],
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: scanlines.as_entire_binding()
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: pixels.as_entire_binding()
					}
				]
			}),
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("floorceil::Data.bind_groups[2]"),
				layout: &bind_group_layouts[2],
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&floor_texview)
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::TextureView(&ceil_texview)
					},
					wgpu::BindGroupEntry {
						binding: 2,
						resource: wgpu::BindingResource::Sampler(&sampler)
					}
				]
			})
		];

		let fillscreen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("floorceil fillscreen shader"),
			source: wgpu::ShaderSource::Wgsl(ShaderSource::FILLSCREEN.into())
		});
		let fpfloorceil_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("floorceil first person floorceil shader"),
			source: wgpu::ShaderSource::Wgsl(ShaderSource::FIRSTPERSON_FLOORCEIL.into())
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("floorceil pipeline layout"),
			bind_group_layouts: &[&bind_group_layouts[0], &bind_group_layouts[1], &bind_group_layouts[2]],
			push_constant_ranges: &[]
		});

		let compute_pipelines = [
			device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("floorceil::Data.compute_pipelines[0]"),
				layout: Some(&pipeline_layout),
				module: &fpfloorceil_shader,
				entry_point: "scanline_process"
			}),
			device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("floorceil::Data.compute_pipelines[1]"),
				layout: Some(&pipeline_layout),
				module: &fpfloorceil_shader,
				entry_point: "pixel_process"
			})
		];

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("floorceil::Data.render_pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &fillscreen_shader,
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
				module: &fpfloorceil_shader,
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
			surface_info, camera_info, tilemap_info, bind_groups, compute_pipelines, render_pipeline,
			_scanlines: scanlines, _pixels: pixels, 
			_floor_texview: floor_texview, _ceil_texview: ceil_texview, _sampler: sampler
		}
	}
}