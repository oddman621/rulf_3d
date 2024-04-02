use std::f32::consts::PI;

use crate::wall;
use crate::webgpu::WebGPU;
use crate::game::GameWorld;

mod floorceil;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SurfaceInfo {
	width: u32,
	half_height: u32,
	scale: u32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RaycastData {
	distance: f32,
	texid: u32,
	u_offset: f32
}

pub struct Renderer {
	wall_render: WallRender,
	depth_texture: wgpu::Texture
}

impl Renderer {
	pub fn render(&mut self, webgpu: &WebGPU, game_world: &GameWorld, clear_color: &wgpu::Color) {
		let output = webgpu.surface.get_current_texture().unwrap();
		let size = output.texture.size();
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		if self.depth_texture.size() != size {
			self.depth_texture.destroy();
			self.depth_texture = webgpu.device.create_texture(&wgpu::TextureDescriptor {
				label: Some("Renderer::depth_texture from render()"),
				mip_level_count: 1,
				sample_count: 1,
				size,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Depth32Float,
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
				view_formats: &[]
			});
		}
		let depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
		
		let surface_info = SurfaceInfo {
			width: output.texture.width(),
			half_height: output.texture.height() / 2,
			scale: 1
		};
		let raycount = 300;
		if let Ok(raycast) = wall::multiple_raycast(
			&game_world.get_walls(), game_world.get_grid_size(), 
			game_world.get_player_position(), game_world.get_player_forward_vector(), 
			PI/2.0, raycount, 50
		) {
			let raycast_data: Vec<RaycastData> = raycast.into_iter().map(|(d, t, u)| RaycastData {
				distance: d, texid: t, u_offset: u
			}).collect();
			self.wall_render.write(&webgpu.queue, surface_info, &raycast_data);

			let mut encoder = webgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
			let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("firstperson::Renderer::render() clearcolor render pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(*clear_color),
						store: wgpu::StoreOp::Store
					}
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
					view: &depth_view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: wgpu::StoreOp::Store
					}),
					stencil_ops: None
				}),
				..Default::default()
			});
			drop(render_pass);

			self.wall_render.draw(&mut encoder, &view, &depth_view);

			webgpu.queue.submit(Some(encoder.finish()));
			output.present();
		}
	}
	pub fn new(webgpu: &WebGPU) -> Self {
		let depth_texture = webgpu.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Renderer::depth_texture"),
			mip_level_count: 1,
			sample_count: 1,
			size: wgpu::Extent3d {
				width: 1, height: 1, depth_or_array_layers: 1
			},
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Depth32Float,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[]
		});
		Self {
			wall_render: WallRender::new(webgpu), depth_texture
		}
	}
}

struct WallRender {
	surface_info_buffer: wgpu::Buffer,
	raycast_data_array_buffer: wgpu::Buffer,
	_texture_array: wgpu::Texture,
	_texture_view: wgpu::TextureView,
	_texture_sampler: wgpu::Sampler,
	bind_groups: [wgpu::BindGroup; 2],
	pipeline: wgpu::RenderPipeline
}

impl WallRender {
	const MAX_RAYCOUNT: u64 = 4320; //8K
}

impl WallRender {
	pub fn write(&mut self, queue: &wgpu::Queue, surface_info: SurfaceInfo, raycast_data: &Vec<RaycastData>) {
		
		queue.write_buffer(&self.surface_info_buffer, 0, bytemuck::cast_slice(&[surface_info]));
		queue.write_buffer(&self.raycast_data_array_buffer, 0, bytemuck::cast_slice(&[raycast_data.len() as u32]));
		queue.write_buffer(&self.raycast_data_array_buffer, std::mem::size_of::<u32>() as u64, bytemuck::cast_slice(raycast_data));
	}
	pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, depth_view: &wgpu::TextureView) {
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("WallRender::draw()"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view, resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Load,
					store: wgpu::StoreOp::Store
				}
			})],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &depth_view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Load,
					store: wgpu::StoreOp::Store
				}),
				stencil_ops: None
			}),
			..Default::default()
		});

		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_bind_group(0, &self.bind_groups[0], &[]);
		render_pass.set_bind_group(1, &self.bind_groups[1], &[]);
		render_pass.draw(0..4, 0..1);
	}
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
		
		let wall_array_image = image::load_from_memory(include_bytes!("asset/all_6.jpg")).unwrap();
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

		for n in 1..=5 {
			for m in 0..=4 {
				webgpu.queue.write_texture(
					wgpu::ImageCopyTexture {
						texture: &texture_array,
						aspect: wgpu::TextureAspect::All,
						mip_level: 0,
						origin: wgpu::Origin3d {
							x: 0, y: 0, z: m,
						}
					}, 
					&image_data, 
					wgpu::ImageDataLayout {
						bytes_per_row: Some(4 * texture_array_size.width * 5),
						rows_per_image: Some(texture_array_size.height),
						offset: (texture_array_size.width * 4 * m) as u64
					},
					wgpu::Extent3d {
							width: texture_array_size.width,
							height: texture_array_size.height,
							depth_or_array_layers: n
					}
				);
			}
		}
		
		let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
			dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default()
		});

		// let wall_array_image = image::load_from_memory(include_bytes!("asset/array_texture.png")).unwrap();
		// let texture_array_size = wgpu::Extent3d {
		// 	width: wall_array_image.width(),
		// 	height: wall_array_image.height() / 4,
		// 	depth_or_array_layers: 4
		// };
		// let texture_array = webgpu.device.create_texture(&wgpu::TextureDescriptor {
		// 	label: Some("WallRender::_texture_array"),
		// 	size: texture_array_size,
		// 	mip_level_count: 1,
		// 	sample_count: 1,
		// 	dimension: wgpu::TextureDimension::D2,
		// 	format: wgpu::TextureFormat::Rgba8UnormSrgb,
		// 	usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		// 	view_formats: &[]
		// });
		// let texture_array_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
		// 	dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default()
		// });

		// let image_data = wall_array_image.to_rgba8();
		// let image_copy_texture = wgpu::ImageCopyTexture {
		// 	texture: &texture_array,
		// 	aspect: wgpu::TextureAspect::All,
		// 	mip_level: 0,
		// 	origin: wgpu::Origin3d::ZERO
		// };
		// let image_data_layout = wgpu::ImageDataLayout {
		// 	bytes_per_row: Some(4 * texture_array_size.width),
		// 	rows_per_image: Some(texture_array_size.height),
		// 	..Default::default()
		// };

		// for n in 1..=4 {
		// 	webgpu.queue.write_texture(image_copy_texture, &image_data, image_data_layout,
		// 		wgpu::Extent3d {
		// 			width: texture_array_size.width,
		// 			height: texture_array_size.height,
		// 			depth_or_array_layers: n
		// 	});
		// }


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
			source: wgpu::ShaderSource::Wgsl(include_str!("asset/fillscreen.wgsl").into())
		});
		let firstperson_wall_shader_module = webgpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("WallRender firstperson wall shader module"),
			source: wgpu::ShaderSource::Wgsl(include_str!("asset/firstperson_wall.wgsl").into())
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
				depth_compare: wgpu::CompareFunction::LessEqual,
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