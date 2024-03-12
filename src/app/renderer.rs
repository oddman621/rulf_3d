use wgpu::BlendState;



#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	position: glam::Vec3,
	color: glam::Vec3,
	uv: glam::Vec2
}

const QUAD_VERTICES: [Vertex; 4] = [
	Vertex { position: glam::vec3(0.5, -0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 1.0) },
	Vertex { position: glam::vec3(0.5, 0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.0) },
	Vertex { position: glam::vec3(-0.5, -0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) },
	Vertex { position: glam::vec3(-0.5, 0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) }
];

const TRIANGLE_VERTICES: [Vertex; 3] = [
	Vertex { position: glam::vec3(0.5, 0.0, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(1.0, 0.5) },
	Vertex { position: glam::vec3(-0.5, 0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 0.0) },
	Vertex { position: glam::vec3(-0.5, -0.5, 0.0), color: glam::Vec3::ONE, uv: glam::vec2(0.0, 1.0) }
];



//TODO: Remove unused fields?
pub struct MiniMapRenderer {
	wall_vb: wgpu::Buffer,
	wall_pos_instb: wgpu::Buffer,
	actor_vb: wgpu::Buffer,
	actor_pos_instb: wgpu::Buffer,
	actor_ang_instb: wgpu::Buffer,
	viewproj_ub: wgpu::Buffer,
	gridsize_ub: wgpu::Buffer,
	color_ub: wgpu::Buffer,
	wall_texture: wgpu::Texture,
	wall_texture_view: wgpu::TextureView,
	texture_sampler: wgpu::Sampler,

	wall_bind_group: wgpu::BindGroup,
	actor_bind_group: wgpu::BindGroup,

	depth_texture: wgpu::Texture,

	wall_render_pipeline: wgpu::RenderPipeline,
	actor_render_pipeline: wgpu::RenderPipeline
}

impl MiniMapRenderer {
	const MAX_WALL_INSTANCE: u64 = 512 * 512;
	const MAX_ACTOR_INSTANCE: u64 = 512;

	pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
		let wall_vb = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("MiniMapRenderer::wall_vb"),
			size: std::mem::size_of_val(&QUAD_VERTICES) as u64,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		queue.write_buffer(&wall_vb, 0, bytemuck::cast_slice(&QUAD_VERTICES));

		let wall_pos_instb = device.create_buffer(&wgpu::BufferDescriptor{
			label: Some(format!("MiniMapRenderer::wall_instb with size:{}", MiniMapRenderer::MAX_WALL_INSTANCE).as_str()),
			size: std::mem::size_of::<[u32; 2]>() as u64 * Self::MAX_WALL_INSTANCE,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let actor_vb = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("MiniMapRenderer::actor_vb"),
			size: std::mem::size_of_val(&TRIANGLE_VERTICES) as u64,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		queue.write_buffer(&actor_vb, 0, bytemuck::cast_slice(&TRIANGLE_VERTICES));

		let actor_pos_instb = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("MiniMapRenderer::actor_pos_instb"),
			size: std::mem::size_of::<[f32; 2]>() as u64 * Self::MAX_ACTOR_INSTANCE,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let actor_ang_instb = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("MiniMapRenderer::actor_ang_instb"),
			size: std::mem::size_of::<[f32; 2]>() as u64 * Self::MAX_ACTOR_INSTANCE,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let viewproj_ub = device.create_buffer(&wgpu::BufferDescriptor{
			label: Some("MiniMapRenderer::viewproj_ub"),
			size: std::mem::size_of::<glam::Mat4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let gridsize_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("MiniMapRenderer::gridsize_ub"),
			size: std::mem::size_of::<[f32; 2]>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});
		let color_ub = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("MiniMapRenderer::color_ub"),
			size: std::mem::size_of::<glam::Vec4>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false
		});

		let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("MiniMapRenderer::depth_texture"),
			size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Depth32Float,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[]
		});

		let wall_image = image::load_from_memory(include_bytes!("Bricks069_32.jpg")).unwrap();
		let wall_image_size = wgpu::Extent3d {
			width: wall_image.width(),
			height: wall_image.height(),
			depth_or_array_layers: 1
		};
		let wall_texture = device.create_texture(&wgpu::TextureDescriptor {
				label: Some("MiniMapRenderer::wall_texture"),
				size: wall_image_size,
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Rgba8UnormSrgb,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				view_formats: &[]
		});
		queue.write_texture(
			wgpu::ImageCopyTexture {
				texture: &wall_texture,
				aspect: wgpu::TextureAspect::All,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO
			}, 
			&wall_image.to_rgba8(), 
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * wall_image_size.width),
				rows_per_image: None
			}, 
			wall_image_size);
		let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("MiniMapRenderer::texture_sampler"),
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});
		let wall_texture_view = wall_texture.create_view(&wgpu::TextureViewDescriptor::default());

		let wall_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("MiniMapRenderer wall bind group layout"),
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
				wgpu::BindGroupLayoutEntry { // Texture for Wall
					binding: 2,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
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

		let actor_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("MiniMapRenderer actor bind group layout"),
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
				wgpu::BindGroupLayoutEntry { //color
					binding: 1,
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

		let wall_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("MiniMapRenderer::wall_bind_group"),
			layout: &wall_bind_group_layout,
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
					resource: wgpu::BindingResource::TextureView(&wall_texture_view)
				},
				wgpu::BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::Sampler(&texture_sampler)
				}
			]
		});

		let actor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("MiniMapRenderer::actor_bind_group"),
			layout: &actor_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: viewproj_ub.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: color_ub.as_entire_binding()
				}
			]
		});

		let wall_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
			label: Some("MiniMapRenderer wall pipeline layout"),
			bind_group_layouts: &[&wall_bind_group_layout],
			push_constant_ranges: &[]
		});
		let actor_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
			label: Some("MiniMapRenderer actor pipeline layout"),
			bind_group_layouts: &[&actor_bind_group_layout],
			push_constant_ranges: &[]
		});

		let wall_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("MiniMapRenderer wall shader module"),
			source: wgpu::ShaderSource::Wgsl(include_str!("wall.wgsl").into())
		});
		let actor_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("MiniMapRenderer actor shader module"),
			source: wgpu::ShaderSource::Wgsl(include_str!("actor.wgsl").into())
		});
		

		let wall_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("MiniMapRenderer::wall_render_pipeline"),
			layout: Some(&wall_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &wall_shader,
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
						array_stride: std::mem::size_of::<[u32; 2]>() as u64,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Uint32x2,
								offset: 0,
								shader_location: 3
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
				module: &wall_shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_format,
					blend: Some(BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				})]
			}),
			multiview: None
		});

		let actor_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
			label: Some("MiniMapRenderer::actor_render_pipeline"),
			layout: Some(&actor_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &actor_shader,
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
						array_stride: std::mem::size_of::<glam::Vec2>() as u64,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32x2,
								offset: 0,
								shader_location: 3
							}
						]
					},
					wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<f32>() as u64,
						step_mode: wgpu::VertexStepMode::Instance,
						attributes: &[
							wgpu::VertexAttribute {
								format: wgpu::VertexFormat::Float32,
								offset: 0,
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
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default()
			}),
			multisample: wgpu::MultisampleState::default(),
			fragment: Some(wgpu::FragmentState {
				module: &actor_shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				})]
			}),
			multiview: None
		});


		Self {
			wall_vb,
			wall_pos_instb,
			actor_vb,
			actor_pos_instb,
			actor_ang_instb,
			viewproj_ub,
			gridsize_ub,
			color_ub,
			wall_texture,
			wall_texture_view,
			texture_sampler,
			wall_bind_group,
			actor_bind_group,
			depth_texture,
			wall_render_pipeline,
			actor_render_pipeline
		}
	}

	pub fn clear(&self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface, 
		color: &wgpu::Color
	) {
		let output = surface.get_current_texture().unwrap();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("MiniMapRenderer::clear()"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(*color),
					store: wgpu::StoreOp::Store
				}
			})],
			..Default::default()
		});

		drop(render_pass);
		queue.submit(Some(encoder.finish()));
		output.present();
	}

	pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface,
		clear_color: &wgpu::Color, viewproj: &glam::Mat4,
		wall_offsets: &[glam::UVec2], gridsize: &glam::Vec2,
		actors_pos: &[glam::Vec2], actors_angle: &[f32], actor_color: &glam::Vec4
	) {
		let output = surface.get_current_texture().unwrap();
		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		if self.depth_texture.size() != size {
			self.depth_texture.destroy();
			self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
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

		queue.write_buffer(&self.viewproj_ub, 0, bytemuck::cast_slice(&[*viewproj]));
		queue.write_buffer(&self.gridsize_ub, 0, bytemuck::cast_slice(&[*gridsize]));
		queue.write_buffer(&self.wall_pos_instb, 0, bytemuck::cast_slice(wall_offsets));

		queue.write_buffer(&self.actor_pos_instb, 0, bytemuck::cast_slice(actors_pos));
		queue.write_buffer(&self.actor_ang_instb, 0, bytemuck::cast_slice(actors_angle));
		queue.write_buffer(&self.viewproj_ub, 0, bytemuck::cast_slice(&[*viewproj]));
		queue.write_buffer(&self.color_ub, 0, bytemuck::cast_slice(&[*actor_color]));

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("MiniMapRenderer::draw_actors()"),
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

		render_pass.set_pipeline(&self.wall_render_pipeline);
		render_pass.set_bind_group(0, &self.wall_bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.wall_vb.slice(..));
		render_pass.set_vertex_buffer(1, self.wall_pos_instb.slice(..));
		render_pass.draw(0..4, 0..wall_offsets.len() as u32);

		render_pass.set_pipeline(&self.actor_render_pipeline);
		render_pass.set_vertex_buffer(0, self.actor_vb.slice(..));
		render_pass.set_vertex_buffer(1, self.actor_pos_instb.slice(..));
		render_pass.set_vertex_buffer(2, self.actor_ang_instb.slice(..));
		render_pass.set_bind_group(0, &self.actor_bind_group, &[]);
		render_pass.draw(0..3, 0..actors_pos.len() as u32);

		drop(render_pass);
		queue.submit(Some(encoder.finish()));
		output.present();
	}
}
