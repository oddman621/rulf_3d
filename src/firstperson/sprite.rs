
use crate::{
	webgpu::{WebGPU, WebGPUDevice, WebGPUConfig},
	asset::AssetServer
};
use super::{SurfaceInfo, Rect};

pub struct BlitData {
	pub surface_info: wgpu::Buffer,
	pub rect: wgpu::Buffer,

	pub bind_groups: [wgpu::BindGroup; 2],
	pub pipeline: wgpu::RenderPipeline,

	_texview: wgpu::TextureView,
	_sampler: wgpu::Sampler
}

impl BlitData {
	pub fn test_blit(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
		let (device, _) = webgpu.get_device();
		let surface_info = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("sprite::BlitData.surface_info"),
			size: std::mem::size_of::<SurfaceInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let rect = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("sprite::BlitData.rect"),
			size: std::mem::size_of::<Rect>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let texture = asset_server.get_texture("buddha16_5x2").unwrap();

		let texview = texture.create_view(&wgpu::TextureViewDescriptor {
			label: Some("sprite::BlitData._texview"),
			dimension: Some(wgpu::TextureViewDimension::D2),
			..Default::default()
		});

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

		let bind_group_layouts = [
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("BlitData bind group layout 0: info buffer"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					}
				]
			}),
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("BlitData bind group layout 1: texture"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
							view_dimension: wgpu::TextureViewDimension::D2,
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
			})
		];

		let bind_groups = [
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("sprite::BlitData.bind_groups[0]"),
				layout: &bind_group_layouts[0],
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: surface_info.as_entire_binding()
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: rect.as_entire_binding()
					}
				]
			}),
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("sprite::BlitData.bind_groups[1]"),
				layout: &bind_group_layouts[1],
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&texview)
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&sampler)
					}
				]
			})
		];

		let shader = asset_server.get_shader("texture_blit").unwrap();

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("sprite BlitData pipeline layout"),
			bind_group_layouts: &[&bind_group_layouts[0], &bind_group_layouts[1]],
			push_constant_ranges: &[]
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("sprite::BlitData.pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[],
				compilation_options: wgpu::PipelineCompilationOptions::default()
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
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: webgpu.get_config().format,
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
					write_mask: wgpu::ColorWrites::all()
				})],
				compilation_options: wgpu::PipelineCompilationOptions::default()
			}),
			multiview: None,
			cache: None
		});

		Self {
			surface_info, rect, bind_groups, pipeline, _texview: texview, _sampler: sampler
		}
	}

	// pub fn new(webgpu: &WebGPU, asset_server: &AssetServer) -> Self {
	// 	let (device, _) = webgpu.get_device();
	// 	let surface_info = device.create_buffer(&wgpu::BufferDescriptor {
	// 		label: Some("sprite::BlitData.surface_info"),
	// 		size: std::mem::size_of::<SurfaceInfo>() as u64,
	// 		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	// 		mapped_at_creation: false
	// 	});
	// 	let rect = device.create_buffer(&wgpu::BufferDescriptor {
	// 		label: Some("sprite::BlitData.rect"),
	// 		size: std::mem::size_of::<Rect>() as u64,
	// 		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	// 		mapped_at_creation: false
	// 	});

	// 	let texture = asset_server.get_texture("buddha16_5x2").unwrap();

	// 	let texview = texture.create_view(&wgpu::TextureViewDescriptor {
	// 		label: Some("sprite::BlitData._texview"),
	// 		dimension: Some(wgpu::TextureViewDimension::D2),
	// 		..Default::default()
	// 	});

	// 	let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

	// 	let bind_group_layouts = [
	// 		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
	// 			label: Some("BlitData bind group layout 0: info buffer"),
	// 			entries: &[
	// 				wgpu::BindGroupLayoutEntry {
	// 					binding: 0,
	// 					visibility: wgpu::ShaderStages::VERTEX,
	// 					ty: wgpu::BindingType::Buffer {
	// 						ty: wgpu::BufferBindingType::Uniform,
	// 						has_dynamic_offset: false,
	// 						min_binding_size: None
	// 					},
	// 					count: None
	// 				},
	// 				wgpu::BindGroupLayoutEntry {
	// 					binding: 1,
	// 					visibility: wgpu::ShaderStages::VERTEX,
	// 					ty: wgpu::BindingType::Buffer {
	// 						ty: wgpu::BufferBindingType::Uniform,
	// 						has_dynamic_offset: false,
	// 						min_binding_size: None
	// 					},
	// 					count: None
	// 				}
	// 			]
	// 		}),
	// 		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
	// 			label: Some("BlitData bind group layout 1: texture"),
	// 			entries: &[
	// 				wgpu::BindGroupLayoutEntry {
	// 					binding: 0,
	// 					visibility: wgpu::ShaderStages::FRAGMENT,
	// 					ty: wgpu::BindingType::Texture {
	// 						sample_type: wgpu::TextureSampleType::Float { filterable: true },
	// 						view_dimension: wgpu::TextureViewDimension::D2,
	// 						multisampled: false
	// 					},
	// 					count: None
	// 				},
	// 				wgpu::BindGroupLayoutEntry {
	// 					binding: 1,
	// 					visibility: wgpu::ShaderStages::FRAGMENT,
	// 					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
	// 					count: None
	// 				}
	// 			]
	// 		})
	// 	];

	// 	let bind_groups = [
	// 		device.create_bind_group(&wgpu::BindGroupDescriptor {
	// 			label: Some("sprite::BlitData.bind_groups[0]"),
	// 			layout: &bind_group_layouts[0],
	// 			entries: &[
	// 				wgpu::BindGroupEntry {
	// 					binding: 0,
	// 					resource: surface_info.as_entire_binding()
	// 				},
	// 				wgpu::BindGroupEntry {
	// 					binding: 1,
	// 					resource: rect.as_entire_binding()
	// 				}
	// 			]
	// 		}),
	// 		device.create_bind_group(&wgpu::BindGroupDescriptor {
	// 			label: Some("sprite::BlitData.bind_groups[1]"),
	// 			layout: &bind_group_layouts[1],
	// 			entries: &[
	// 				wgpu::BindGroupEntry {
	// 					binding: 0,
	// 					resource: wgpu::BindingResource::TextureView(&texview)
	// 				},
	// 				wgpu::BindGroupEntry {
	// 					binding: 1,
	// 					resource: wgpu::BindingResource::Sampler(&sampler)
	// 				}
	// 			]
	// 		})
	// 	];

	// 	let shader = asset_server.get_shader("texture_blit").unwrap();

	// 	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
	// 		label: Some("sprite BlitData pipeline layout"),
	// 		bind_group_layouts: &[&bind_group_layouts[0], &bind_group_layouts[1]],
	// 		push_constant_ranges: &[]
	// 	});

	// 	let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
	// 		label: Some("sprite::BlitData.pipeline"),
	// 		layout: Some(&pipeline_layout),
	// 		vertex: wgpu::VertexState {
	// 			module: &shader,
	// 			entry_point: "vs_main",
	// 			buffers: &[],
	// 			compilation_options: wgpu::PipelineCompilationOptions::default()
	// 		},
	// 		primitive: wgpu::PrimitiveState {
	// 			topology: wgpu::PrimitiveTopology::TriangleStrip,
	// 			strip_index_format: None,
	// 			front_face: wgpu::FrontFace::Ccw,
	// 			cull_mode: Some(wgpu::Face::Back),
	// 			unclipped_depth: false,
	// 			polygon_mode: wgpu::PolygonMode::Fill,
	// 			conservative: false
	// 		},
	// 		depth_stencil: Some(wgpu::DepthStencilState {
	// 			format: wgpu::TextureFormat::Depth32Float,
	// 			depth_write_enabled: true,
	// 			depth_compare: wgpu::CompareFunction::LessEqual,
	// 			stencil: wgpu::StencilState::default(),
	// 			bias: wgpu::DepthBiasState::default()
	// 		}),
	// 		multisample: wgpu::MultisampleState::default(),
	// 		fragment: Some(wgpu::FragmentState {
	// 			module: &shader,
	// 			entry_point: "fs_main",
	// 			targets: &[Some(wgpu::ColorTargetState {
	// 				format: webgpu.get_config().format,
	// 				blend: Some(wgpu::BlendState::ALPHA_BLENDING),
	// 				write_mask: wgpu::ColorWrites::all()
	// 			})],
	// 			compilation_options: wgpu::PipelineCompilationOptions::default()
	// 		}),
	// 		multiview: None,
	// 		cache: None
	// 	});

	// 	Self {
	// 		surface_info, rect, bind_groups, pipeline, _texview: texview, _sampler: sampler
	// 	}
	// }
}
