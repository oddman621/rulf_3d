use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
 
pub struct AssetServer {
    shaders: HashMap<&'static str, wgpu::ShaderModule>,
    images: HashMap<&'static str, image::DynamicImage>,
	textures: HashMap<&'static str, wgpu::Texture>
}

pub enum ArrayOrder {
	_Column, Row
}

pub enum TextureType {
	_Full,
	_Partial {
		x: u32,
		y: u32,
		width: u32,
		height: u32
	},
	Grid {
		order: ArrayOrder,
		x: u32,
		y: u32
	},
	_PaddingGrid {
		order: ArrayOrder,
		x: u32,
		y: u32,
		left: u32,
		right: u32,
		top: u32,
		bottom: u32
	}
}
 
pub enum AssetServerError {
    DuplicatedTextureName, 
	OpenFileFailed(std::io::Error), 
	ReadImageFailed(image::ImageError), 
	ImageNameNotFound,
	InvalidGridLength,
	InvalidPosition,
	InvalidSize
}

impl AssetServer {
	pub fn create_test_asset_server(device: &wgpu::Device, queue: &wgpu::Queue) -> AssetServer {
		const SHADER_SOURCES: [(&'static str, &'static str); 6] = [
			("fillscreen", "asset/fillscreen.wgsl"),
			("firstperson_wall_compute", "asset/firstperson_wall_compute.wgsl"),
			("firstperson_wall_frag", "asset/firstperson_wall_frag.wgsl"),
			("firstperson_floorceil", "asset/firstperson_floorceil.wgsl"),
			("minimap_actor", "asset/minimap_actor.wgsl"),
			("minimap_wall", "asset/minimap_wall.wgsl"),
		];
		const IMAGES: [(&'static str, &'static str); 2] = [
			("all_6", "asset/all_6.jpg"),
			("buddha_16x16", "asset/buddha_16x16.png")
		];

		let mut asset_server = AssetServer::new();

		for (name, path) in SHADER_SOURCES {
			if let Err(_) = asset_server.load_wgsl(device, name, path) {
				panic!("Failed to load {}", path);
			}
		}

		for (name, path) in IMAGES {
			if let Err(_) = asset_server.load_image(name, path) {
				panic!("Failed to load {}", path);
			}
		}

		if let Err(_) = asset_server.create_image_texture(
			device, queue, "all_6", "all_6_5x5",
			&TextureType::Grid { 
				order: ArrayOrder::Row,
				x: 5, y: 5
			}
		) {
			panic!("Failed to create texture all_6");
		}

		if let Err(_) = asset_server.create_image_texture(
			device, queue, "buddha_16x16", "buddha16_5x2",
			&TextureType::Grid {
				order: ArrayOrder::Row,
				x: 5, y: 2
			} 
		) {
			panic!("Failed to create texture buddha16_5x2(buddha_16x16)")
		}

		asset_server
	}
}
 
impl AssetServer {
    pub fn new() -> AssetServer {
        AssetServer {
            shaders: HashMap::new(),
            images: HashMap::new(),
			textures: HashMap::new()
        }
    }
    fn load_str(path: &Path) -> io::Result<String> {
        let mut file = match File::open(path) {
            Err(e) => return Err(e),
            Ok(f) => f
        };
 
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(e) => return Err(e),
            Ok(_) => return Ok(s)
        }
    }
 
    fn load_byte(path: &Path) -> io::Result<Vec<u8>> {
        let mut file = match File::open(path) {
            Err(e) => return Err(e),
            Ok(f) => f
        };
        let mut b = Vec::<u8>::new();
        match file.read_to_end(&mut b) {
            Err(e) => return Err(e),
            Ok(_) => return Ok(b)
        }
    }
 
    pub fn load_wgsl(&mut self, device: &wgpu::Device, name: &'static str, path: &str) -> Result<(), AssetServerError> {
        if self.shaders.contains_key(name) {
            return Err(AssetServerError::DuplicatedTextureName);
        }
 
        let string = match Self::load_str(Path::new(path)) {
            Ok(s) => s,
            Err(e) => return Err(AssetServerError::OpenFileFailed(e))
        };
 
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(name),
            source: wgpu::ShaderSource::Wgsl(string.as_str().into())
        });
 
        self.shaders.insert(name, shader);
 
        Ok(())
    }
 
    pub fn load_image(&mut self, name: &'static str, path: &str) -> Result<(), AssetServerError> {
        if self.images.contains_key(name) {
            return Err(AssetServerError::DuplicatedTextureName);
        }
 
        let bytes = match Self::load_byte(Path::new(path)) {
            Ok(b) => b,
            Err(e) => return Err(AssetServerError::OpenFileFailed(e))
        };
 
        let image = match image::load_from_memory(bytes.as_slice()) {
            Ok(img) => img,
            Err(e) => return Err(AssetServerError::ReadImageFailed(e))
        };
 
        self.images.insert(name, image);
 
        Ok(())
    }

	pub fn get_texture(&self, name: &str) -> Option<&wgpu::Texture> {
		self.textures.get(name)
	}

	pub fn get_shader(&self, name: &str) -> Option<&wgpu::ShaderModule> {
		self.shaders.get(name)
	}

	pub fn create_image_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, image_name: &'static str, name: &'static str, textype: &TextureType) -> Result<(), AssetServerError> {
		if self.textures.contains_key(name) {
			return Err(AssetServerError::DuplicatedTextureName);
		}

		let image = match self.images.get(image_name) {
			None => return Err(AssetServerError::ImageNameNotFound),
			Some(img) => img
		};
		let data = image.to_rgba8();

		match textype {
			TextureType::_Full => { // NOTE: Not Tested
				let size = wgpu::Extent3d {
					width: image.width(),
					height: image.height(),
					depth_or_array_layers: 1
				};
				let texture = device.create_texture(&wgpu::TextureDescriptor {
					label: Some(name),
					size,
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Rgba8UnormSrgb,
					usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
					view_formats: &[]
				});
				queue.write_texture(
					wgpu::ImageCopyTexture {
						texture: &texture,
						mip_level: 0,
						origin: wgpu::Origin3d {
							x:0, y:0, z:0
						},
						aspect: wgpu::TextureAspect::All
					}, 
					&data,
					wgpu::ImageDataLayout {
						offset: 0,
						bytes_per_row: Some(size.width * 4),
						rows_per_image: Some(size.height)
					},
					size
				);
				self.textures.insert(name, texture);
				Ok(())
			},
			TextureType::Grid { order, x, y } => {
					let length = x * y;
					if length == 0 {
						return Err(AssetServerError::InvalidGridLength);
					}
					let size = wgpu::Extent3d {
						width: image.width() / x,
						height: image.height() / y,
						depth_or_array_layers: length
					};
					let texture = device.create_texture(&wgpu::TextureDescriptor {
						label: Some(name),
						size,
						mip_level_count: 1,
						sample_count: 1,
						dimension: wgpu::TextureDimension::D2,
						format: wgpu::TextureFormat::Rgba8UnormSrgb,
						usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
						view_formats: &[]
					});
	
					let offset = |i| match order {
						ArrayOrder::Row => (size.width * (i % x) + size.width * x * size.height * (i / x)) * 4,
						ArrayOrder::_Column => (size.width * x * size.height * (i % x) + size.width * (i / x)) * 4
					};
	
					for l in 0..length {
						queue.write_texture(
							wgpu::ImageCopyTexture {
								texture: &texture,
								aspect: wgpu::TextureAspect::All,
								mip_level: 0,
								origin: wgpu::Origin3d {
									x: 0, y: 0, z: l
								}
							},
							&data,
							wgpu::ImageDataLayout {
								offset: offset(l) as u64,
								bytes_per_row: Some(4 * size.width * x),
								rows_per_image: Some(size.height)
							},
							wgpu::Extent3d {
								width: size.width,
								height: size.height,
								depth_or_array_layers: 1
							}
						)
					};
	
					self.textures.insert(name, texture);
					Ok(())
				},
			TextureType::_PaddingGrid { order, x, y, left, right, top, bottom } => {
				let length = x * y;
				if length == 0 {
					return Err(AssetServerError::InvalidGridLength);
				}
				let size = wgpu::Extent3d {
					width: image.width() / x - (left + right),
					height: image.height() / y - (bottom + top),
					depth_or_array_layers: length
				};
				let texture = device.create_texture(&wgpu::TextureDescriptor {
					label: Some(name),
					size,
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Rgba8UnormSrgb,
					usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
					view_formats: &[]
				});

				let gridsize_width = image.width() / x;
				let gridsize_height = image.height() / y;
				let offset = |i| match order {
					ArrayOrder::Row => (gridsize_width * (i % x) + gridsize_width * x * gridsize_height * (i / x)) * 4,
					ArrayOrder::_Column => (gridsize_width * x * gridsize_height * (i % x) + gridsize_width * (i / x)) * 4
				};
				let grid_offset = (image.width() * top + left) * 4;
				for l in 0..length {
					queue.write_texture(
						wgpu::ImageCopyTexture {
							texture: &texture,
							aspect: wgpu::TextureAspect::All,
							mip_level: 0,
							origin: wgpu::Origin3d {
								x: 0, y: 0, z: l
							}
						},
						&data,
						wgpu::ImageDataLayout {
							offset: offset(l) as u64 + grid_offset as u64,
							bytes_per_row: Some(4 * image.width()),
							rows_per_image: Some(image.height())
						},
						wgpu::Extent3d {
							width: size.width,
							height: size.height,
							depth_or_array_layers: 1
						}
					)
				};

				self.textures.insert(name, texture);
				Ok(())
			},
			TextureType::_Partial { x, y, width, height } => { // NOTE: Not Tested
				if *x as i32 == 0 || *y as i32 == 0 {
					return Err(AssetServerError::InvalidPosition);
				}
				if x + width >= image.width() || y + height >= image.height() {
					return Err(AssetServerError::InvalidSize)
				}
				let size = wgpu::Extent3d {
					width: *width,
					height: *height,
					depth_or_array_layers: 1
				};

				let texture = device.create_texture(&wgpu::TextureDescriptor {
					label: Some(name),
					size,
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: wgpu::TextureFormat::Rgba8UnormSrgb,
					usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
					view_formats: &[]
				});
				let grid_offset = (image.width() * y + x) * 4;
				queue.write_texture(
					wgpu::ImageCopyTexture {
						texture: &texture,
						mip_level: 0,
						origin: wgpu::Origin3d {
							x: 0, y: 0, z:0
						},
						aspect: wgpu::TextureAspect::All
					}, 
					&data,
					wgpu::ImageDataLayout {
						offset: grid_offset as u64,
						bytes_per_row: Some(image.width() * 4),
						rows_per_image: Some(image.height())
					},
					size
				);
				self.textures.insert(name, texture);
				Ok(())
			}
		}
	}
}
 
#[test]
fn asset_server_functionality() {
    use crate::webgpu::MinimalWebGPU;
	use crate::webgpu::WebGPUDevice;
    let webgpu = MinimalWebGPU::_new();
	let (device, _) = webgpu.get_device();
    let mut asset_server =  AssetServer::new();
 
    assert!(asset_server.load_image("testimg", "src/asset/all_6.jpg").is_ok());
    assert!(asset_server.load_wgsl(device, "test", "src/asset/fillscreen.wgsl").is_ok());
}