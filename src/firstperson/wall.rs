
use crate::{
	webgpu::WebGPU,
	asset::{ShaderSource, ImageByte}
};

use super::{SurfaceInfo, RaycastData};
pub struct Data {
	pub surface_info_buffer: wgpu::Buffer,
	pub raycast_data_array_buffer: wgpu::Buffer,
	_texture_array: wgpu::Texture,
	_texture_view: wgpu::TextureView,
	_texture_sampler: wgpu::Sampler,
	pub bind_groups: [wgpu::BindGroup; 2],
	pub pipeline: wgpu::RenderPipeline
}

impl Data {
	const MAX_RAYCOUNT: u64 = 4320; //8K
}

impl Data {
	pub fn new(webgpu: &WebGPU) -> Self {
		let surface_info_buffer = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::surface_info_buffer"),
			size: std::mem::size_of::<SurfaceInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let raycast_data_array_buffer = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::raycast_data_array_buffer"),
			size: std::mem::size_of::<u32>() as u64 +  
				std::mem::size_of::<RaycastData>() as u64 * Self::MAX_RAYCOUNT,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let texture_sampler = webgpu.device.create_sampler(&wgpu::SamplerDescriptor::default());
		
		let wall_array_image = image::load_from_memory(ImageByte::ALL_6).unwrap();
		let texture_array_size = wgpu::Extent3d {
			width: wall_array_image.width() / 5,
			height: wall_array_image.height() / 5,
			depth_or_array_layers: 25
		};
		let texture_array = webgpu.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("WallRender::_texture_array"),
			size: texture_array_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[]
		});

		let image_data = wall_array_image.to_rgba8();

		for layer in 1..=5 {
			for offset in 0..=4 {
				webgpu.queue.write_texture(
					wgpu::ImageCopyTexture {
						texture: &texture_array,
						aspect: wgpu::TextureAspect::All,
						mip_level: 0,
						origin: wgpu::Origin3d {
							x: 0, y: 0, z: offset,
						}
					}, 
					&image_data, 
					wgpu::ImageDataLayout {
						bytes_per_row: Some(4 * texture_array_size.width * 5),
						rows_per_image: Some(texture_array_size.height),
						offset: (texture_array_size.width * 4 * offset) as u64
					},
					wgpu::Extent3d {
							width: texture_array_size.width,
							height: texture_array_size.height,
							depth_or_array_layers: layer
					}
				);
			}
		}
		
		let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
			dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default()
		});

		let bind_group_0_layout = webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

		let bind_group_1_layout = webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

		let bind_groups = [
			webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
			webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

		let pipeline_layout = webgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("WallRender pipeline layout"),
			bind_group_layouts: &[&bind_group_0_layout, &bind_group_1_layout],
			..Default::default()
		});

		let fillscreen_shader_module = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("WallRender fillscreen shader module"),
			source: wgpu::ShaderSource::Wgsl(ShaderSource::FILLSCREEN.into())
		});
		let firstperson_wall_shader_module = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("WallRender firstperson wall shader module"),
			source: wgpu::ShaderSource::Wgsl(ShaderSource::FIRSTPERSON_WALL.into())
		});

		let pipeline = webgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("WallRender::pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &fillscreen_shader_module,
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
				depth_compare: wgpu::CompareFunction::Always, //FIXME: If LessEqual, walls will not drawn. That is not intended.
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default()
			}),
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: &firstperson_wall_shader_module,
				entry_point: "main",
				targets: &[Some(wgpu::ColorTargetState {
					format: webgpu.config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				})]
			}),
			multiview: None
		});

		Self {
			surface_info_buffer, raycast_data_array_buffer,
			_texture_array: texture_array,
			_texture_view: texture_array_view,
			_texture_sampler: texture_sampler,
			bind_groups, pipeline
		}
	}
}