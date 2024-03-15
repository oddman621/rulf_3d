use wgpu::util::DeviceExt;

use super::WebGPU;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: glam::Vec3,
	color: glam::Vec3,
	uv: glam::Vec2
}

pub struct Renderer {
	depth_texture: wgpu::Texture,
	wall_render: WallRender,
	actor_render: ActorRender
}

impl Renderer {
	pub fn new(webgpu: &WebGPU) -> Self {
		let depth_texture = webgpu.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Renderer::depth_texture"),
			size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Depth32Float,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[]
		});

		Self { depth_texture,wall_render: WallRender::new(webgpu), actor_render: ActorRender::new(webgpu) }
	}

	pub fn draw(&mut self, webgpu: &WebGPU,
		clear_color: &wgpu::Color, viewproj: &glam::Mat4,
		walls: &[u32], gridsize: &glam::Vec2,
		actors_pos: &[glam::Vec2], actors_angle: &[f32], actor_size: f32, actor_color: &glam::Vec4
	) {
		webgpu.queue.write_buffer(&self.actor_render.viewproj_ub, 0, bytemuck::cast_slice(&[*viewproj]));
		webgpu.queue.write_buffer(&self.wall_render.viewproj_ub, 0, bytemuck::cast_slice(&[*viewproj]));
		webgpu.queue.write_buffer(&self.wall_render.gridsize_ub, 0, bytemuck::cast_slice(&[*gridsize]));
		webgpu.queue.write_buffer(&self.actor_render.actorsize_ub, 0, bytemuck::cast_slice(&[actor_size]));
		webgpu.queue.write_buffer(&self.actor_render.color_ub, 0, bytemuck::cast_slice(&[*actor_color]));

		webgpu.queue.write_buffer(&self.wall_render.instb, 0, bytemuck::cast_slice(walls));
		if actors_pos.len() == actors_angle.len() {
			let actor_inst_data: Vec<f32> = actors_pos.into_iter().enumerate()
				.map(|(idx, pos)| [pos.x, pos.y, actors_angle[idx]]).flatten().collect();
			webgpu.queue.write_buffer(&self.actor_render.instb, 0, bytemuck::cast_slice(&actor_inst_data));
		}

		let output = webgpu.surface.get_current_texture().unwrap();
		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		if self.depth_texture.size() != size {
			self.depth_texture.destroy();
			self.depth_texture = webgpu.device.create_texture(&wgpu::TextureDescriptor {
				label: Some(format!("MiniMapRenderer::depth_texture ({}, {})", size.width, size.height).as_str()),
				size: wgpu::Extent3d {
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
			});
		}
		let depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
		
		let mut encoder = webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Renderer::draw()"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(*clear_color),
					store: wgpu::StoreOp::Store
				}
			})],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
				view: &depth_texture_view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: wgpu::StoreOp::Store
				}),
				stencil_ops: None
			}),
			..Default::default()
		});

		render_pass.set_pipeline(&self.wall_render.pipeline);
		render_pass.set_bind_group(0, &self.wall_render.bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.wall_render.vb.slice(..));
		render_pass.set_vertex_buffer(1, self.wall_render.instb.slice(..));
		render_pass.draw(0..4, 0..(walls.len() as u32 / 3));

		render_pass.set_pipeline(&self.actor_render.pipeline);
		render_pass.set_bind_group(0, &self.actor_render.bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.actor_render.vb.slice(..));
		render_pass.set_vertex_buffer(1, self.actor_render.instb.slice(..));
		render_pass.draw(0..3, 0..actors_pos.len() as u32);

		drop(render_pass);
		webgpu.queue.submit(Some(encoder.finish()));
		output.present();
	}
}

struct WallRender {
	vb: wgpu::Buffer,
	instb: wgpu::Buffer,
	viewproj_ub: wgpu::Buffer,
	gridsize_ub: wgpu::Buffer,
	_texture_array: wgpu::Texture,
	_texture_array_view: wgpu::TextureView,
	_texture_sampler: wgpu::Sampler,
	bind_group: wgpu::BindGroup,
	pipeline: wgpu::RenderPipeline,
}

impl WallRender {
	const MAX_WALL_INSTANCE: u64 = 512 * 512;
	const QUAD_VERTICES: [Vertex; 4] = [
		Vertex { position: glam::vec3(1.0, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 1.0) },
		Vertex { position: glam::vec3(1.0, 1.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.0) },
		Vertex { position: glam::vec3(0.0, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) },
		Vertex { position: glam::vec3(0.0, 1.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) }
	];
}

struct ActorRender {

	vb: wgpu::Buffer,
	instb: wgpu::Buffer,
	viewproj_ub: wgpu::Buffer,
	color_ub: wgpu::Buffer,
	actorsize_ub: wgpu::Buffer,

	bind_group: wgpu::BindGroup,
	pipeline: wgpu::RenderPipeline
}

impl ActorRender {
	const MAX_ACTOR_INSTANCE: u64 = 512;
	const TRIANGLE_VERTICES: [Vertex; 3] = [
		Vertex { position: glam::vec3(0.5, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.5) },
		Vertex { position: glam::vec3(-0.5, 0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) },
		Vertex { position: glam::vec3(-0.5, -0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) }
	];
}

impl WallRender {
	fn new(webgpu: &WebGPU) -> Self {
		let vb = webgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("WallRender::vb"),
			contents: bytemuck::cast_slice(&Self::QUAD_VERTICES),
			usage: wgpu::BufferUsages::VERTEX
		});

		let instb = webgpu.device.create_buffer(&wgpu::BufferDescriptor {// offset: [u32;2], texid: u32
			label: Some("WallRender::instb"),
			size: Self::MAX_WALL_INSTANCE * std::mem::size_of::<u32>() as u64 * 3,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let viewproj_ub = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::viewproj_ub"),
			size: std::mem::size_of::<glam::Mat4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let gridsize_ub = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("WallRender::gridsize_ub"),
			size: std::mem::size_of::<f32>() as u64 * 2,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let texture_sampler = webgpu.device.create_sampler(&wgpu::SamplerDescriptor::default());
		
		let wall_array_image = image::load_from_memory(include_bytes!("array_texture.png")).unwrap();
		let texture_array_size = wgpu::Extent3d {
			width: wall_array_image.width(),
			height: wall_array_image.height() / 4,
			depth_or_array_layers: 4
		};
		//TODO: create_texture_with_data로도 texture array를 작성할 수 있나?
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
		let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
			dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default()
		});

		
		let image_data = wall_array_image.to_rgba8();
		let image_copy_texture = wgpu::ImageCopyTexture {
			texture: &texture_array,
			aspect: wgpu::TextureAspect::All,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO
		};
		let image_data_layout = wgpu::ImageDataLayout {
			bytes_per_row: Some(4 * texture_array_size.width),
			rows_per_image: Some(texture_array_size.height),
			//offset: 0, // WTF: wgpu::ImageDataLayout.offset seems be ommitted when 2 or more queue.write_texture() to same texture_array. Instead, height field must be Some.
			..Default::default()
		};
		webgpu.queue.write_texture(image_copy_texture, &image_data, image_data_layout,
			wgpu::Extent3d {
				width: texture_array_size.width,
				height: texture_array_size.height,
				depth_or_array_layers: 1
		});
		webgpu.queue.write_texture(image_copy_texture, &image_data, image_data_layout,
			wgpu::Extent3d {
				width: texture_array_size.width,
				height: texture_array_size.height,
				depth_or_array_layers: 2
		});
		webgpu.queue.write_texture(image_copy_texture, &image_data, image_data_layout,
			wgpu::Extent3d {
				width: texture_array_size.width,
				height: texture_array_size.height,
				depth_or_array_layers: 3
		});
		webgpu.queue.write_texture(image_copy_texture, &image_data, image_data_layout,
			wgpu::Extent3d {
				width: texture_array_size.width,
				height: texture_array_size.height,
				depth_or_array_layers: 4
		});

		let bind_group_layout = webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

		let bind_group = webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

		let pipeline_layout = webgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("WallRender pipeline layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[]
		});

		let shader_module = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("WallRender wall.wgsl shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("wall.wgsl").into())
		});
		let render_pipeline = webgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("WallRender::render_pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_module,
				entry_point: "vs_main",
				buffers: &[
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as u64,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x3,
								offset: 0,
								shader_location: 0
							},
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x3,
								offset: std::mem::size_of::<glam::Vec3>() as u64,
								shader_location: 1
							},
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x2,
								offset: std::mem::size_of::<glam::Vec3>() as u64 * 2,
								shader_location: 2
							}
						]
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
			depth_stencil: Some(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: true,
				stencil: wgpu::StencilState::default(),
				depth_compare: wgpu::CompareFunction::Less,
				bias: wgpu::DepthBiasState::default()
			}),
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: &shader_module,
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
			vb, instb, viewproj_ub, gridsize_ub, bind_group, pipeline: render_pipeline,
			_texture_array: texture_array, _texture_array_view: texture_array_view, _texture_sampler: texture_sampler
		}
	}
	fn draw(webgpu: &WebGPU) {
		todo!()
	}
}



impl ActorRender {
	fn new(webgpu: &WebGPU) -> Self {
		let vb = webgpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("ActorRender::vb"),
			usage: wgpu::BufferUsages::VERTEX,
			contents: bytemuck::cast_slice(&Self::TRIANGLE_VERTICES)
		});

		let instb = webgpu.device.create_buffer(&wgpu::BufferDescriptor { //pos: [f32;2], ang: f32
			label: Some("ActorRender::instb"),
			size: std::mem::size_of::<f32>() as u64 * 3 * Self::MAX_ACTOR_INSTANCE,
			usage:wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let viewproj_ub = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ActorRender::viewproj_ub"),
			size: std::mem::size_of::<glam::Mat4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		
		let actorsize_ub = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ActorRender::actorsize_ub"),
			size: std::mem::size_of::<f32>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let color_ub = webgpu.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ActorRender::color_ub"),
			size: std::mem::size_of::<glam::Vec4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let bind_group_layout = webgpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

		let bind_group = webgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

		let pipeline_layout = webgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("ActorRender pipeline layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[]
		});
		let shader_module = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("ActorRender actor.wgsl shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("actor.wgsl").into())
		});

		let render_pipeline = webgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
			label: Some("ActorRender::render_pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_module,
				entry_point: "vs_main",
				buffers: &[
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as u64,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x3,
								offset: 0,
								shader_location: 0
							},
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x3,
								offset: std::mem::size_of::<glam::Vec3>() as u64,
								shader_location: 1
							},
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x2,
								offset: std::mem::size_of::<glam::Vec3>() as u64 * 2,
								shader_location: 2
							}
						]
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
			depth_stencil: Some(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::LessEqual,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default()
			}),
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: &shader_module,
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
			vb, instb, viewproj_ub, color_ub, actorsize_ub, bind_group, pipeline: render_pipeline
		}
	}
}


