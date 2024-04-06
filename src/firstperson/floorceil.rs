use crate::{
	webgpu::WebGPU, 
	asset::{ImageByte, ShaderSource}
};
use super::{SurfaceInfo, FloorCeilCameraInfo};

pub struct Data {
	pub surface_info: wgpu::Buffer,
	pub camera_info: wgpu::Buffer,
	pub tilemap_info: wgpu::Buffer,

	pub bind_groups: [wgpu::BindGroup; 3],
	pub compute_pipelines: [wgpu::ComputePipeline; 2],
	pub render_pipeline: wgpu::RenderPipeline,

	_scanlines: wgpu::Buffer,
	_pixels: wgpu::Buffer,

	_floor_texarray: wgpu::Texture,
	_ceil_texarray: wgpu::Texture,
	_floor_texview: wgpu::TextureView,
	_ceil_texview: wgpu::TextureView,
	_sampler: wgpu::Sampler	
}

impl Data {
	const MAXIMUM_TILEMAP_SIZE: u64 = 2048 * 2048;
	const MAX_WIDTH: u64 = 3840;
	const MAX_HEIGHT: u64 = 2160;

	const SCANLINEDATA_SIZE: u64 = std::mem::size_of::<glam::Vec2>() as u64 * 2;
	const PIXELINFO_SIZE: u64 = std::mem::size_of::<glam::IVec2>() as u64 + std::mem::size_of::<glam::Vec2>() as u64;
}

impl Data {
	pub fn new(webgpu: &WebGPU) -> Self {
		let surface_info = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data.surface_info"),
			size: std::mem::size_of::<SurfaceInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let camera_info = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data.camera_info"),
			size: std::mem::size_of::<FloorCeilCameraInfo>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let tilemap_info = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data.tilemap_info"),
			size: std::mem::size_of::<glam::UVec2>() as u64 + std::mem::size_of::<glam::IVec2>() as u64 * Self::MAXIMUM_TILEMAP_SIZE,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let scanlines = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data._scanlines"),
			size: Self::SCANLINEDATA_SIZE * Self::MAX_HEIGHT,
			usage: wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});
		let pixels = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("floorceil::Data._pixels"),
			size: Self::PIXELINFO_SIZE * Self::MAX_WIDTH * Self::MAX_HEIGHT,
			usage: wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});

		let array_image = image::load_from_memory(ImageByte::ALL_6).unwrap();
		let imgdata = array_image.to_rgba8();
		let texarray_size = wgpu::Extent3d {
			width: array_image.width() / 5,
			height: array_image.height() / 5,
			depth_or_array_layers: 25
		};
		let floor_texarray = webgpu.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("floorceil::Data._floor_texarray"),
			size: texarray_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[]
		});
		let ceil_texarray = webgpu.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("floorceil::Data._ceil_texarray"),
			size: texarray_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[]
		});

		let texgrid_offset_bytes = glam::uvec2(4 * texarray_size.width, 4 * texarray_size.width * 5 * texarray_size.height);
		for l in 0..25 {
			let origin = wgpu::Origin3d { x: 0, y: 0, z: l};
			let data_layout = wgpu::ImageDataLayout {
				bytes_per_row: Some(4 * texarray_size.width * 5),
				rows_per_image: Some(texarray_size.height),
				offset: (texgrid_offset_bytes.x * (l % 5) + texgrid_offset_bytes.y * (l / 5)) as u64
			};
			let size = wgpu::Extent3d {
				width: texarray_size.width,
				height: texarray_size.height,
				depth_or_array_layers: 1
			};

			webgpu.queue.write_texture(
				wgpu::ImageCopyTexture {
					texture: &floor_texarray,
					aspect: wgpu::TextureAspect::All,
					mip_level: 0,
					origin
				}, 
				&imgdata, data_layout, size
			);
			webgpu.queue.write_texture(
				wgpu::ImageCopyTexture {
					texture: &ceil_texarray,
					aspect: wgpu::TextureAspect::All,
					mip_level: 0,
					origin
				},
				&imgdata, data_layout, size
			);
		}

		let floor_texview = floor_texarray.create_view(&wgpu::TextureViewDescriptor {
			label: Some("floorceil::Data._floor_texview"),
			dimension: Some(wgpu::TextureViewDimension::D2Array),
			..Default::default()
		});
		let ceil_texview = ceil_texarray.create_view(&wgpu::TextureViewDescriptor {
			label: Some("floorceil::Data._ceil_texview"),
			dimension: Some(wgpu::TextureViewDimension::D2Array),
			..Default::default()
		});

		let sampler = webgpu.device.create_sampler(&wgpu::SamplerDescriptor::default());

		let bind_group_layouts = [
			webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
			webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
			webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
			webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
			webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
			webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

		let fillscreen_shader = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("floorceil fillscreen shader"),
			source: wgpu::ShaderSource::Wgsl(ShaderSource::FILLSCREEN.into())
		});
		let fpfloorceil_shader = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("floorceil first person floorceil shader"),
			source: wgpu::ShaderSource::Wgsl(ShaderSource::FIRSTPERSON_FLOORCEIL.into())
		});

		let pipeline_layout = webgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("floorceil pipeline layout"),
			bind_group_layouts: &[&bind_group_layouts[0], &bind_group_layouts[1], &bind_group_layouts[2]],
			push_constant_ranges: &[]
		});

		let compute_pipelines = [
			webgpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("floorceil::Data.compute_pipelines[0]"),
				layout: Some(&pipeline_layout),
				module: &fpfloorceil_shader,
				entry_point: "scanline_process"
			}),
			webgpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("floorceil::Data.compute_pipelines[1]"),
				layout: Some(&pipeline_layout),
				module: &fpfloorceil_shader,
				entry_point: "pixel_process"
			})
		];

		let render_pipeline = webgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
					format: webgpu.config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				})]
			}),
			multiview: None
		});

		Self {
			surface_info, camera_info, tilemap_info, bind_groups, compute_pipelines, render_pipeline,
			_scanlines: scanlines, _pixels: pixels, _floor_texarray: floor_texarray,
			_ceil_texarray: ceil_texarray, _floor_texview: floor_texview,
			_ceil_texview: ceil_texview, _sampler: sampler
		}
	}
}